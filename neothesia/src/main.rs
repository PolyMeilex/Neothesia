#![allow(clippy::collapsible_match, clippy::single_match)]

mod iced_utils;
mod input_manager;
mod output_manager;
mod scene;
mod song;
mod target;
mod utils;

use std::time::Duration;

use iced_core::Renderer;
use scene::{menu_scene, playing_scene, Scene};
use target::Target;
use utils::window::WindowState;

use midi_file::midly::MidiMessage;
use neothesia_core::{config, render};
use wgpu_jumpstart::Surface;
use wgpu_jumpstart::{Gpu, TransformUniform};
use winit::{
    event::WindowEvent,
    event_loop::{EventLoop, EventLoopBuilder},
};

#[derive(Debug)]
pub enum NeothesiaEvent {
    /// Go to playing scene
    Play(song::Song),
    /// Go to main menu scene
    MainMenu,
    MidiInput {
        /// The MIDI channel that this message is associated with.
        channel: u8,
        /// The MIDI message type and associated data.
        message: MidiMessage,
    },
    Exit,
}

struct Neothesia {
    target: Target,
    surface: Surface,
    game_scene: Box<dyn Scene>,

    #[cfg(debug_assertions)]
    fps_ticker: fps_ticker::Fps,
    last_time: std::time::Instant,
}

impl Neothesia {
    fn new(mut target: Target, surface: Surface) -> Self {
        let mut game_scene = menu_scene::MenuScene::new(&mut target);

        target.resize();
        game_scene.resize(&mut target);
        target.gpu.submit();

        Self {
            target,
            surface,
            game_scene: Box::new(game_scene),

            #[cfg(debug_assertions)]
            fps_ticker: fps_ticker::Fps::default(),
            last_time: std::time::Instant::now(),
        }
    }

    fn window_event(
        &mut self,
        event: &WindowEvent,
        event_loop: &winit::event_loop::EventLoopWindowTarget<NeothesiaEvent>,
    ) {
        self.target.window_state.window_event(event);

        match &event {
            WindowEvent::Resized(_) => {
                self.surface.resize_swap_chain(
                    &self.target.gpu.device,
                    self.target.window_state.physical_size.width,
                    self.target.window_state.physical_size.height,
                );

                self.target.resize();
                self.game_scene.resize(&mut self.target);

                self.target.gpu.submit();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // TODO: Check if this update is needed;
                self.target.resize();
                self.game_scene.resize(&mut self.target);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        state: winit::event::ElementState::Pressed,
                        logical_key,
                        ..
                    },
                ..
            } => match logical_key {
                winit::keyboard::Key::Character(c) if c.as_str() == "f" => {
                    if self.target.window.fullscreen().is_some() {
                        self.target.window.set_fullscreen(None);
                    } else {
                        let monitor = self.target.window.current_monitor();
                        if let Some(monitor) = monitor {
                            let f = winit::window::Fullscreen::Borderless(Some(monitor));
                            self.target.window.set_fullscreen(Some(f));
                        } else {
                            let f = winit::window::Fullscreen::Borderless(None);
                            self.target.window.set_fullscreen(Some(f));
                        }
                    }
                }
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                let delta = self.last_time.elapsed();
                self.last_time = std::time::Instant::now();

                self.update(delta);
                self.render();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        self.game_scene.window_event(&mut self.target, event);
    }

    fn neothesia_event(
        &mut self,
        event: NeothesiaEvent,
        event_loop: &winit::event_loop::EventLoopWindowTarget<NeothesiaEvent>,
    ) {
        match event {
            NeothesiaEvent::Play(song) => {
                self.target.iced_manager.renderer.clear();

                let to = playing_scene::PlayingScene::new(&self.target, song);
                self.game_scene = Box::new(to);
            }
            NeothesiaEvent::MainMenu => {
                let to = menu_scene::MenuScene::new(&mut self.target);
                self.game_scene = Box::new(to);
            }
            NeothesiaEvent::MidiInput { channel, message } => {
                self.game_scene
                    .midi_event(&mut self.target, channel, &message);
            }
            NeothesiaEvent::Exit => {
                event_loop.exit();
            }
        }
    }

    fn update(&mut self, delta: Duration) {
        #[cfg(debug_assertions)]
        {
            self.fps_ticker.tick();
            self.target.text_renderer.queue_fps(self.fps_ticker.avg());
        }

        self.game_scene.update(&mut self.target, delta);
        self.target.text_renderer.update(
            self.target.window_state.logical_size.into(),
            &self.target.gpu,
        );
    }

    fn render(&mut self) {
        let frame = loop {
            let swap_chain_output = self.surface.get_current_texture();
            match swap_chain_output {
                Ok(s) => break s,
                Err(err) => log::warn!("{:?}", err),
            }
        };

        let view = &frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let bg_color = self.target.config.background_color;
            let bg_color = wgpu_jumpstart::Color::from(bg_color).into_linear_wgpu_color();
            let mut rpass =
                self.target
                    .gpu
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Main Neothesia Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(bg_color),
                                store: wgpu::StoreOp::Store,
                            },
                        })],

                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

            self.game_scene.render(&self.target.transform, &mut rpass);
            self.target.text_renderer.render(&mut rpass);
        }

        self.target
            .iced_manager
            .renderer
            .with_primitives(|backend, primitive| {
                if !primitive.is_empty() {
                    backend.present(
                        &self.target.gpu.device,
                        &self.target.gpu.queue,
                        &mut self.target.gpu.encoder,
                        None,
                        self.target.gpu.texture_format,
                        view,
                        primitive,
                        &self.target.iced_manager.viewport,
                        &self.target.iced_manager.debug.overlay(),
                    );
                }
            });

        self.target.gpu.submit();
        frame.present();

        self.target.text_renderer.atlas().trim();
    }
}

fn main() {
    let builder = winit::window::WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
        width: 1080.0,
        height: 720.0,
    });

    let (event_loop, target, surface) = init(builder);

    let mut app = Neothesia::new(target, surface);

    // Investigate:
    // https://github.com/gfx-rs/wgpu-rs/pull/306

    event_loop
        .run(move |event, event_loop| {
            use winit::event::Event;
            match event {
                Event::UserEvent(event) => {
                    app.neothesia_event(event, event_loop);
                }
                Event::WindowEvent { event, .. } => {
                    app.window_event(&event, event_loop);
                }
                Event::AboutToWait => {
                    app.target.window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}

fn init(builder: winit::window::WindowBuilder) -> (EventLoop<NeothesiaEvent>, Target, Surface) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("neothesia=info"))
        .init();

    let event_loop = EventLoopBuilder::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    let builder = builder
        .with_title("Neothesia")
        .with_theme(Some(winit::window::Theme::Dark));

    #[cfg(target_os = "windows")]
    let builder = {
        use winit::platform::windows::WindowBuilderExtWindows;
        builder.with_drag_and_drop(false)
    };

    let window = builder.build(&event_loop).unwrap();

    if let Err(err) = set_window_icon(&window) {
        log::error!("Failed to load window icon: {}", err);
    }

    let window_state = WindowState::new(&window);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu_jumpstart::default_backends(),
        dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        flags: wgpu::InstanceFlags::default(),
        gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
    });

    let size = window.inner_size();
    let (gpu, surface) =
        futures::executor::block_on(Gpu::for_window(&instance, &window, size.width, size.height))
            .unwrap();

    let target = Target::new(window, window_state, proxy, gpu);

    (event_loop, target, surface)
}

fn set_window_icon(window: &winit::window::Window) -> Result<(), Box<dyn std::error::Error>> {
    use iced_graphics::image::image_rs;
    use image_rs::codecs::png::PngDecoder;
    use image_rs::ImageDecoder;
    use std::io::Cursor;

    let icon = PngDecoder::new(Cursor::new(include_bytes!(
        "../../flatpak/com.github.polymeilex.neothesia.png"
    )))?;

    let (w, h) = icon.dimensions();

    let mut buff = vec![0; icon.total_bytes() as usize];
    icon.read_image(&mut buff)?;

    window.set_window_icon(Some(winit::window::Icon::from_rgba(buff, w, h)?));

    Ok(())
}
