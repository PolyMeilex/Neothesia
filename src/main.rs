#![cfg(feature = "app")]

use neothesia::{
    scene::{self, Scene, SceneEvent, SceneType},
    target::Target,
    utils::timer::Fps,
};

use winit::{event::WindowEvent, event_loop::ControlFlow};

pub struct Neothesia {
    pub target: Target,

    pub fps_timer: Fps,
    pub game_scene: scene::scene_transition::SceneTransition,
}

impl Neothesia {
    pub fn new(mut target: Target) -> Self {
        let game_scene = scene::menu_scene::MenuScene::new(&mut target);
        let mut game_scene =
            scene::scene_transition::SceneTransition::new(Box::new(game_scene), &target);

        target.resize();
        game_scene.resize(&mut target);
        target.gpu.submit().unwrap();

        Self {
            target,
            fps_timer: Fps::new(),
            game_scene,
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        match &event {
            WindowEvent::Resized(_) => {
                self.target.resize();
                self.game_scene.resize(&mut self.target);

                self.target.gpu.submit().unwrap();
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
                    let f = if let Some(monitor) = monitor {
                        let modes: Vec<_> = monitor.video_modes().collect();

                        log::info!(
                            "Avalible Video Modes: {:#?}",
                            modes.iter().map(|m| format!("{}", m)).collect::<Vec<_>>()
                        );

                        if let Some(m) = modes.into_iter().next() {
                            log::info!("Selected Video Mode #{}: {}", 0, m);
                            winit::window::Fullscreen::Exclusive(m)
                        } else {
                            winit::window::Fullscreen::Borderless(None)
                        }
                    } else {
                        winit::window::Fullscreen::Borderless(None)
                    };

                    self.target.window.set_fullscreen(Some(f));
                }
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        }

        let scene_event = self.game_scene.window_event(&mut self.target, event);
        self.scene_event(scene_event, control_flow);
    }

    pub fn scene_event(&mut self, event: SceneEvent, control_flow: &mut ControlFlow) {
        match event {
            SceneEvent::MainMenu(event) => match event {
                scene::menu_scene::Event::Play => {
                    let to = |target: &mut Target| -> Box<dyn Scene> {
                        let state = scene::playing_scene::PlayingScene::new(target);
                        Box::new(state)
                    };

                    let to = Box::new(to);

                    self.game_scene.transition_to(to);
                }
            },
            SceneEvent::GoBack => match self.game_scene.scene_type() {
                SceneType::MainMenu => {
                    *control_flow = ControlFlow::Exit;
                }
                SceneType::Playing => {
                    let to = |target: &mut Target| -> Box<dyn Scene> {
                        let state = scene::menu_scene::MenuScene::new(target);
                        Box::new(state)
                    };

                    let to = Box::new(to);

                    self.game_scene.transition_to(to);
                }
                SceneType::Transition => {}
            },
            _ => {}
        }
    }

    pub fn update(&mut self, control_flow: &mut ControlFlow) {
        self.fps_timer.update();

        let event = self.game_scene.update(&mut self.target);

        self.scene_event(event, control_flow);

        #[cfg(debug_assertions)]
        self.target.text_renderer.queue_fps(self.fps_timer.fps());
    }

    pub fn render(&mut self) {
        let frame = loop {
            let swap_chain_output = self.target.window.get_current_frame();
            match swap_chain_output {
                Ok(s) => break s,
                Err(err) => log::warn!("{:?}", err),
            }
        };

        self.target.gpu.clear(
            &frame.output.view,
            self.target.state.config.background_color.into(),
        );

        self.game_scene.render(&mut self.target, &frame.output.view);

        self.target.text_renderer.render(
            &self.target.window,
            &mut self.target.gpu,
            &frame.output.view,
        );

        self.target.gpu.submit().unwrap();
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
            Event::MainEventsCleared => {
                let event = app.game_scene.main_events_cleared(&mut app.target);
                app.scene_event(event, control_flow);

                app.target.window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.window_event(event, control_flow);
            }
            Event::RedrawRequested(_) => {
                app.update(control_flow);
                app.render();
            }
            _ => {}
        }
    });
}
