#![allow(clippy::collapsible_match, clippy::single_match)]

mod context;
mod iced_utils;
mod input_manager;
mod output_manager;
mod scene;
mod song;
mod utils;

use std::sync::Arc;
use std::time::Duration;

use context::Context;
use iced_core::Renderer;
use scene::{menu_scene, playing_scene, Scene};
use utils::window::WindowState;

use midi_file::midly::MidiMessage;
use neothesia_core::{config, render};
use wgpu_jumpstart::Surface;
use wgpu_jumpstart::{Gpu, TransformUniform};
use winit::application::ApplicationHandler;
use winit::event_loop::EventLoopProxy;
use winit::{event::WindowEvent, event_loop::EventLoop};

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
    context: Context,
    surface: Surface,
    game_scene: Box<dyn Scene>,

    #[cfg(debug_assertions)]
    fps_ticker: fps_ticker::Fps,
    last_time: std::time::Instant,
}

impl Neothesia {
    fn new(mut context: Context, surface: Surface) -> Self {
        let game_scene = menu_scene::MenuScene::new(&mut context);

        context.resize();
        context.gpu.submit();

        Self {
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
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        self.context.window_state.window_event(&event);

        match &event {
            WindowEvent::Resized(_) => {
                self.surface.resize_swap_chain(
                    &self.context.gpu.device,
                    self.context.window_state.physical_size.width,
                    self.context.window_state.physical_size.height,
                );

                self.context.resize();

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
                profiling::finish_frame!();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }

        self.game_scene.window_event(&mut self.context, &event);
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: NeothesiaEvent,
    ) {
        match event {
            NeothesiaEvent::Play(song) => {
                self.context.iced_manager.renderer.clear();

                let to = playing_scene::PlayingScene::new(&self.context, song);
                self.game_scene = Box::new(to);
            }
            NeothesiaEvent::MainMenu => {
                let to = menu_scene::MenuScene::new(&mut self.context);
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

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.context.window.request_redraw();
    }

    #[profiling::function]
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

    #[profiling::function]
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

        self.context.iced_manager.renderer.present(
            &mut self.context.iced_manager.engine,
            &self.context.gpu.device,
            &self.context.gpu.queue,
            &mut self.context.gpu.encoder,
            None,
            self.context.gpu.texture_format,
            view,
            &self.context.iced_manager.viewport,
            &self.context.iced_manager.debug.overlay(),
        );

        let encoder = self.context.gpu.take();
        self.context
            .iced_manager
            .engine
            .submit(&self.context.gpu.queue, encoder);
        // self.context.gpu.submit();
        frame.present();

        self.context.text_renderer.atlas().trim();
    }
}

// This is so stupid, but winit holds us at gunpoint with create_window deprecation
struct NeothesiaBootstrap(Option<Neothesia>, EventLoopProxy<NeothesiaEvent>);

impl ApplicationHandler<NeothesiaEvent> for NeothesiaBootstrap {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.0.is_some() {
            return;
        }

        let mut attributes = winit::window::Window::default_attributes()
            .with_inner_size(winit::dpi::LogicalSize {
                width: 1080.0,
                height: 720.0,
            })
            .with_title("Neothesia")
            .with_theme(Some(winit::window::Theme::Dark));

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            use winit::platform::{
                startup_notify::{
                    self, EventLoopExtStartupNotify, WindowAttributesExtStartupNotify,
                },
                wayland::WindowAttributesExtWayland,
            };

            if let Some(token) = event_loop.read_token_from_env() {
                startup_notify::reset_activation_token_env();
                attributes = attributes.with_activation_token(token);
            }

            attributes = attributes.with_name("com.github.polymeilex.neothesia", "main");
        };

        let window = event_loop.create_window(attributes).unwrap();

        if let Err(err) = set_window_icon(&window) {
            log::error!("Failed to load window icon: {}", err);
        }

        let window_state = WindowState::new(&window);
        let size = window.inner_size();
        let window = Arc::new(window);
        let (gpu, surface) =
            futures::executor::block_on(Gpu::for_window(window.clone(), size.width, size.height))
                .unwrap();

        let ctx = Context::new(window, window_state, self.1.clone(), gpu);

        let app = Neothesia::new(ctx, surface);
        self.0 = Some(app);
    }

    fn user_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        event: NeothesiaEvent,
    ) {
        if let Some(app) = self.0.as_mut() {
            app.user_event(event_loop, event);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        if let Some(app) = self.0.as_mut() {
            app.window_event(event_loop, window_id, event)
        }
    }

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if let Some(app) = self.0.as_mut() {
            app.about_to_wait(event_loop)
        }
    }
}

fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn, wgpu_hal=error, oxisynth=error"),
    )
    .init();

    puffin::set_scopes_on(true); // tell puffin to collect data
    let _server = puffin_http::Server::new("127.0.0.1:8585").ok();

    let event_loop: EventLoop<NeothesiaEvent> = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    event_loop
        .run_app(&mut NeothesiaBootstrap(None, proxy))
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
