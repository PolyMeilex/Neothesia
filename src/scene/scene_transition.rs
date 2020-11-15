use crate::{
    rectangle_pipeline::RectangleInstance,
    scene::{Scene, SceneEvent, SceneType},
    Target,
};

use winit::event::WindowEvent;

enum TransitionMode {
    FadeIn(Box<dyn Scene>),
    FadeOut(Box<dyn Scene>, Box<dyn Scene>),
    Static(Box<dyn Scene>),
    None,
}

pub struct SceneTransition {
    active: bool,
    n: f32,
    mode: TransitionMode,
}

impl SceneTransition {
    pub fn new(game_scene: Box<dyn Scene>) -> Self {
        Self {
            active: true,
            n: 0.0,
            mode: TransitionMode::FadeIn(game_scene),
        }
    }
    pub fn transition_to(&mut self, game_scene: Box<dyn Scene>) {
        let from = std::mem::replace(&mut self.mode, TransitionMode::None);
        match from {
            TransitionMode::Static(scene) => {
                self.mode = TransitionMode::FadeOut(scene, game_scene);
            }
            _ => unreachable!("Trans_to triggered while fade is in progress"),
        };
    }
}

impl Scene for SceneTransition {
    fn scene_type(&self) -> SceneType {
        match &self.mode {
            TransitionMode::Static(scene) => scene.scene_type(),
            _ => SceneType::Transition,
        }
    }
    fn resize(&mut self, target: &mut Target) {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.resize(target),
            TransitionMode::FadeIn(scene) => scene.resize(target),
            TransitionMode::FadeOut(from, to) => {
                from.resize(target);
                to.resize(target);
            }
            _ => {}
        }
    }
    fn update(&mut self, target: &mut Target) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.update(target),
            TransitionMode::FadeIn(scene) => {
                scene.update(target);

                let mut alpha = 1.0 - self.n;

                self.n += 0.03;
                if self.n >= 1.0 {
                    self.n = 0.0;
                    self.active = false;

                    let next = std::mem::replace(&mut self.mode, TransitionMode::None);

                    let mut game_scene = if let TransitionMode::FadeIn(from) = next {
                        from
                    } else {
                        unreachable!("Expected Fade In")
                    };
                    game_scene.start();
                    self.mode = TransitionMode::Static(game_scene);

                    alpha = 0.0;
                }

                let (window_w, window_h) = {
                    let winit::dpi::LogicalSize { width, height } =
                        target.window.state.logical_size;
                    (width, height)
                };
                target.ui.set_transition_alpha(
                    &mut target.gpu,
                    RectangleInstance {
                        color: [0.0, 0.0, 0.0, alpha],
                        size: [window_w, window_h],
                        position: [0.0, 0.0],
                    },
                );
                SceneEvent::None
            }
            TransitionMode::FadeOut(from, _to) => {
                from.update(target);

                let alpha = 0.0 + self.n;

                self.n += 0.03;
                if self.n >= 1.0 {
                    self.n = 0.0;
                    self.active = false;

                    let next = std::mem::replace(&mut self.mode, TransitionMode::None);

                    let game_scene = if let TransitionMode::FadeOut(_from, to) = next {
                        to
                    } else {
                        unreachable!("Expected Fade Out")
                    };
                    self.mode = TransitionMode::FadeIn(game_scene);
                }

                let (window_w, window_h) = {
                    let winit::dpi::LogicalSize { width, height } =
                        target.window.state.logical_size;
                    (width, height)
                };
                target.ui.set_transition_alpha(
                    &mut target.gpu,
                    RectangleInstance {
                        color: [0.0, 0.0, 0.0, alpha],
                        size: [window_w, window_h],
                        position: [0.0, 0.0],
                    },
                );
                SceneEvent::None
            }
            _ => SceneEvent::None,
        }
    }
    fn render(&mut self, target: &mut Target, frame: &wgpu::SwapChainFrame) {
        match &mut self.mode {
            TransitionMode::FadeIn(scene) => scene.render(target, frame),
            TransitionMode::FadeOut(from, _to) => from.render(target, frame),
            TransitionMode::Static(scene) => scene.render(target, frame),
            _ => {}
        }
    }
    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.window_event(target, event),
            _ => SceneEvent::None,
        }
    }
    fn main_events_cleared(&mut self, target: &mut Target) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::FadeIn(scene) => scene.main_events_cleared(target),
            TransitionMode::FadeOut(from, _to) => from.main_events_cleared(target),
            TransitionMode::Static(scene) => scene.main_events_cleared(target),
            _ => SceneEvent::None,
        }
    }
}
