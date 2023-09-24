#![allow(clippy::collapsible_match, clippy::single_match)]

mod iced_utils;
mod input_manager;
mod output_manager;
mod scene;
mod target;
mod utils;

use scene::{menu_scene, playing_scene, Scene};
use target::Target;
use utils::window::WindowState;

use midi_file::midly::MidiMessage;
use neothesia_core::{config, render};
use wgpu_jumpstart::Surface;
use wgpu_jumpstart::{Gpu, TransformUniform};
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
};

#[derive(Debug)]
pub enum NeothesiaEvent {
    /// Go to playing scene
    Play(midi_file::MidiFile),
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

    last_time: std::time::Instant,
    fps_timer: fps_ticker::Fps,
    game_scene: Box<dyn Scene>,
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
            last_time: std::time::Instant::now(),
            fps_timer: Default::default(),
            game_scene: Box::new(game_scene),
        }
    }

    fn window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
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
                input:
                    winit::event::KeyboardInput {
                        state: winit::event::ElementState::Pressed,
                        virtual_keycode: Some(winit::event::VirtualKeyCode::F),
                        ..
                    },
                ..
            } => {
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
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        }

        self.game_scene.window_event(&mut self.target, event);
    }

    fn neothesia_event(&mut self, event: NeothesiaEvent, control_flow: &mut ControlFlow) {
        match event {
            NeothesiaEvent::Play(midi_file) => {
                let to = playing_scene::PlayingScene::new(&self.target, midi_file);
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
                *control_flow = ControlFlow::Exit;
            }
        }
    }

    fn update(&mut self) {
        self.fps_timer.tick();

        let delta = self.last_time.elapsed();
        self.last_time = std::time::Instant::now();

        self.game_scene.update(&mut self.target, delta);

        #[cfg(debug_assertions)]
        self.target.text_renderer.queue_fps(self.fps_timer.avg());
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

        self.target
            .gpu
            .clear(view, self.target.config.background_color.into());

        self.game_scene.render(&mut self.target, view);

        self.target.text_renderer.render(
            (
                self.target.window_state.logical_size.width,
                self.target.window_state.logical_size.height,
            ),
            &mut self.target.gpu,
            view,
        );

        self.target.gpu.submit();
        frame.present();
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

    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
        match event {
            Event::UserEvent(event) => {
                app.neothesia_event(event, control_flow);
            }
            Event::MainEventsCleared => {
                app.game_scene.main_events_cleared(&mut app.target);

                app.update();
                app.target.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.window_event(&event, control_flow);
            }
            Event::RedrawRequested(_) => {
                app.render();
            }
            _ => {}
        }
    });
}

fn init(builder: winit::window::WindowBuilder) -> (EventLoop<NeothesiaEvent>, Target, Surface) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("neothesia=info"))
        .init();

    let event_loop = EventLoopBuilder::with_user_event().build();
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

    let window_state = WindowState::new(&window);
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu_jumpstart::default_backends(),
        dx12_shader_compiler: wgpu::Dx12Compiler::default(),
    });

    let size = window.inner_size();
    let (gpu, surface) =
        futures::executor::block_on(Gpu::for_window(&instance, &window, size.width, size.height))
            .unwrap();

    let target = Target::new(window, window_state, proxy, gpu);

    (event_loop, target, surface)
}
