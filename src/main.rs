#![cfg(feature = "app")]

use neothesia::{
    midi_event::MidiEvent,
    scene::{menu_scene, playing_scene, scene_manager, SceneType},
    target::Target,
    NeothesiaEvent,
};

use winit::{event::WindowEvent, event_loop::ControlFlow};

pub struct Neothesia {
    pub target: Target,

    last_time: std::time::Instant,
    pub fps_timer: fps_ticker::Fps,
    pub game_scene: scene_manager::SceneManager,
}

impl Neothesia {
    pub fn new(mut target: Target) -> Self {
        let game_scene = menu_scene::MenuScene::new(&mut target);
        let mut game_scene = scene_manager::SceneManager::new(game_scene);

        target.resize();
        game_scene.resize(&mut target);
        target.gpu.submit();

        Self {
            target,
            last_time: std::time::Instant::now(),
            fps_timer: Default::default(),
            game_scene,
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match &event {
            WindowEvent::Resized(_) => {
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

    pub fn midi_event(&mut self, event: &MidiEvent) {
        self.game_scene.midi_event(&mut self.target, event);
    }

    pub fn neothesia_event(&mut self, event: &NeothesiaEvent, control_flow: &mut ControlFlow) {
        match event {
            NeothesiaEvent::MainMenu(event) => match event {
                menu_scene::Event::Play => {
                    let to = playing_scene::PlayingScene::new(&mut self.target);
                    self.game_scene.transition_to(&mut self.target, to);
                }
            },
            NeothesiaEvent::GoBack => match self.game_scene.scene_type() {
                SceneType::MainMenu => {
                    *control_flow = ControlFlow::Exit;
                }
                SceneType::Playing => {
                    let to = menu_scene::MenuScene::new(&mut self.target);
                    self.game_scene.transition_to(&mut self.target, to);
                }
            },
            NeothesiaEvent::MidiInput(event) => self.midi_event(event),
        }
    }

    pub fn update(&mut self) {
        self.fps_timer.tick();

        let delta = self.last_time.elapsed();
        self.last_time = std::time::Instant::now();

        self.game_scene.update(&mut self.target, delta);

        #[cfg(debug_assertions)]
        self.target.text_renderer.queue_fps(self.fps_timer.avg());
    }

    pub fn render(&mut self) {
        let frame = loop {
            let swap_chain_output = self.target.window.get_current_texture();
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

        self.target
            .text_renderer
            .render(&self.target.window, &mut self.target.gpu, view);

        self.target.gpu.submit();
        frame.present();
    }
}

fn main() {
    let builder = winit::window::WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
        width: 1080.0,
        height: 720.0,
    });

    let (event_loop, target) = neothesia::init(builder);

    let mut app = Neothesia::new(target);

    // Investigate:
    // https://github.com/gfx-rs/wgpu-rs/pull/306

    event_loop.run(move |event, _, control_flow| {
        app.target.window.on_event(&mut app.target.gpu, &event);

        use winit::event::Event;
        match &event {
            Event::UserEvent(event) => {
                app.neothesia_event(event, control_flow);
            }
            Event::MainEventsCleared => {
                app.game_scene.main_events_cleared(&mut app.target);

                app.update();
                app.target.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.window_event(event, control_flow);
            }
            Event::RedrawRequested(_) => {
                app.render();
            }
            _ => {}
        }
    });
}
