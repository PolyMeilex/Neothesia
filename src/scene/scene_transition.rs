use crate::{
    rectangle_pipeline::RectangleInstance,
    scene::{InputEvent, Scene, SceneEvent, SceneType},
    wgpu_jumpstart::Gpu,
    MainState, Ui,
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
    fn resize(&mut self, state: &mut MainState, gpu: &mut Gpu) {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.resize(state, gpu),
            TransitionMode::FadeIn(scene) => scene.resize(state, gpu),
            TransitionMode::FadeOut(from, to) => {
                from.resize(state, gpu);
                to.resize(state, gpu);
            }
            _ => {}
        }
    }
    fn update(&mut self, state: &mut MainState, gpu: &mut Gpu, ui: &mut Ui) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.update(state, gpu, ui),
            TransitionMode::FadeIn(scene) => {
                scene.update(state, gpu, ui);

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

                ui.set_transition_alpha(
                    gpu,
                    RectangleInstance {
                        color: [0.0, 0.0, 0.0, alpha],
                        size: [state.window_size.0, state.window_size.1],
                        position: [state.window_size.0 / 2.0, state.window_size.1 / 2.0],
                    },
                );
                SceneEvent::None
            }
            TransitionMode::FadeOut(from, _to) => {
                from.update(state, gpu, ui);

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

                ui.set_transition_alpha(
                    gpu,
                    RectangleInstance {
                        color: [0.0, 0.0, 0.0, alpha],
                        size: [state.window_size.0, state.window_size.1],
                        position: [state.window_size.0 / 2.0, state.window_size.1 / 2.0],
                    },
                );
                SceneEvent::None
            }
            _ => SceneEvent::None,
        }
    }
    fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
        match &mut self.mode {
            TransitionMode::FadeIn(scene) => scene.render(state, gpu, frame),
            TransitionMode::FadeOut(from, _to) => from.render(state, gpu, frame),
            TransitionMode::Static(scene) => scene.render(state, gpu, frame),
            _ => {}
        }
    }
    fn input_event(&mut self, state: &mut MainState, event: InputEvent) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.input_event(state, event),
            _ => SceneEvent::None,
        }
    }
    fn window_event(&mut self, state: &mut MainState, event: &WindowEvent) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.window_event(state, event),
            _ => SceneEvent::None,
        }
    }
    fn main_events_cleared(&mut self, state: &mut MainState) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::FadeIn(scene) => scene.main_events_cleared(state),
            TransitionMode::FadeOut(from, _to) => from.main_events_cleared(state),
            TransitionMode::Static(scene) => scene.main_events_cleared(state),
            _ => SceneEvent::None,
        }
    }
}
