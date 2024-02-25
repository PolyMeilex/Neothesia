#![allow(clippy::collapsible_match, clippy::single_match)]

mod context;
mod iced_utils;
mod input_manager;
mod output_manager;
mod scene;
mod song;
mod utils;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use context::Context;
use iced_core::Renderer;
use iced_utils::IcedManager;
use scene::{menu_scene, playing_scene, Scene};
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
    iced_manager: Rc<RefCell<IcedManager>>,
    context: Context,
    surface: Surface,
    game_scene: Box<dyn Scene>,

    #[cfg(debug_assertions)]
    fps_ticker: fps_ticker::Fps,
    last_time: std::time::Instant,
}

impl Neothesia {
    fn new(mut context: Context, surface: Surface) -> Self {
        let iced_manager = Rc::new(RefCell::new(IcedManager::new(
            &context.gpu.device,
            &context.gpu.queue,
            context.gpu.texture_format,
            (
                context.window_state.physical_size.width,
                context.window_state.physical_size.height,
            ),
            context.window_state.scale_factor,
        )));

        let game_scene = menu_scene::MenuScene::new(&mut context, iced_manager.clone());

        context.resize();
        context.gpu.submit();

        Self {
            iced_manager,
            context,
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
        self.context.window_state.window_event(event);

        match &event {
            WindowEvent::Resized(_) => {
                self.surface.resize_swap_chain(
                    &self.context.gpu.device,
                    self.context.window_state.physical_size.width,
                    self.context.window_state.physical_size.height,
                );

                self.context.resize();
                self.iced_manager.borrow_mut().resize(
                    (
                        self.context.window_state.physical_size.width,
                        self.context.window_state.physical_size.height,
                    ),
                    self.context.window_state.scale_factor,
                );

                self.context.gpu.submit();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // TODO: Check if this update is needed;
                self.context.resize();
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
                    if self.context.window.fullscreen().is_some() {
                        self.context.window.set_fullscreen(None);
                    } else {
                        let monitor = self.context.window.current_monitor();
                        if let Some(monitor) = monitor {
                            let f = winit::window::Fullscreen::Borderless(Some(monitor));
                            self.context.window.set_fullscreen(Some(f));
                        } else {
                            let f = winit::window::Fullscreen::Borderless(None);
                            self.context.window.set_fullscreen(Some(f));
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

        self.game_scene.window_event(&mut self.context, event);
    }

    fn neothesia_event(
        &mut self,
        event: NeothesiaEvent,
        event_loop: &winit::event_loop::EventLoopWindowTarget<NeothesiaEvent>,
    ) {
        match event {
            NeothesiaEvent::Play(song) => {
                self.iced_manager.borrow_mut().renderer.clear();

                let to = playing_scene::PlayingScene::new(&self.context, song);
                self.game_scene = Box::new(to);
            }
            NeothesiaEvent::MainMenu => {
                let to = menu_scene::MenuScene::new(&mut self.context, self.iced_manager.clone());
                self.game_scene = Box::new(to);
            }
            NeothesiaEvent::MidiInput { channel, message } => {
                self.game_scene
                    .midi_event(&mut self.context, channel, &message);
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
            self.context.text_renderer.queue_fps(self.fps_ticker.avg());
        }

        self.game_scene.update(&mut self.context, delta);
        self.context.text_renderer.update(
            self.context.window_state.logical_size.into(),
            &self.context.gpu,
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
            let bg_color = self.context.config.background_color;
            let bg_color = wgpu_jumpstart::Color::from(bg_color).into_linear_wgpu_color();
            let mut rpass =
                self.context
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

            self.game_scene.render(&self.context.transform, &mut rpass);
            self.context.text_renderer.render(&mut rpass);
        }

        {
            let iced_manager = &mut *self.iced_manager.borrow_mut();
            iced_manager.renderer.with_primitives(|backend, primitive| {
                if !primitive.is_empty() {
                    backend.present(
                        &self.context.gpu.device,
                        &self.context.gpu.queue,
                        &mut self.context.gpu.encoder,
                        None,
                        self.context.gpu.texture_format,
                        view,
                        primitive,
                        &iced_manager.viewport,
                        &iced_manager.debug.overlay(),
                    );
                }
            });
        }

        self.context.gpu.submit();
        frame.present();

        self.context.text_renderer.atlas().trim();
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("neothesia=info"))
        .init();

    let event_loop: EventLoop<NeothesiaEvent> =
        EventLoopBuilder::with_user_event().build().unwrap();

    let builder = winit::window::WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize {
            width: 1080.0,
            height: 720.0,
        })
        .with_title("Neothesia")
        .with_theme(Some(winit::window::Theme::Dark));

    // TODO: This can be removed now
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
    let size = window.inner_size();
    let window = Arc::new(window);
    let (gpu, surface) =
        futures::executor::block_on(Gpu::for_window(window.clone(), size.width, size.height))
            .unwrap();

    let ctx = Context::new(window, window_state, event_loop.create_proxy(), gpu);

    let mut app = Neothesia::new(ctx, surface);

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
                    app.context.window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
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
