use crate::rectangle_pipeline::RectanglePipeline;
use crate::Gpu;
use crate::{
    rectangle_pipeline::RectangleInstance,
    scene::{Scene, SceneEvent, SceneType},
    target::Target,
};

use winit::event::WindowEvent;

pub type SceneInitializer = dyn FnOnce(&mut Target) -> Box<dyn Scene>;

enum TransitionMode {
    FadeIn(Box<dyn Scene>),
    FadeOut(Box<dyn Scene>, Box<SceneInitializer>),
    Static(Box<dyn Scene>),
    None,
}

pub struct SceneTransition {
    active: bool,
    n: f32,
    mode: TransitionMode,

    transition_pipeline: RectanglePipeline,
    curr_transition_alpha: f32,
}

impl SceneTransition {
    pub fn new(game_scene: Box<dyn Scene>, target: &Target) -> Self {
        let transition_pipeline = RectanglePipeline::new(&target.gpu, &target.transform_uniform);
        Self {
            active: true,
            n: 0.0,
            mode: TransitionMode::FadeIn(game_scene),
            transition_pipeline,
            curr_transition_alpha: 0.0,
        }
    }
    pub fn transition_to(&mut self, initer: Box<SceneInitializer>) {
        let from = std::mem::replace(&mut self.mode, TransitionMode::None);
        match from {
            TransitionMode::Static(scene) => {
                self.mode = TransitionMode::FadeOut(scene, initer);
            }
            _ => unreachable!("Trans_to triggered while fade is in progress"),
        };
    }
    // pub fn transition_to(&mut self, game_scene: Box<dyn Scene>) {
    //     let from = std::mem::replace(&mut self.mode, TransitionMode::None);
    //     match from {
    //         TransitionMode::Static(scene) => {
    //             self.mode = TransitionMode::FadeOut(scene, game_scene);
    //         }
    //         _ => unreachable!("Trans_to triggered while fade is in progress"),
    //     };
    // }
    pub fn set_transition_alpha(
        &mut self,
        gpu: &mut Gpu,
        alpha: f32,
        window_w: f32,
        window_h: f32,
    ) {
        self.curr_transition_alpha = alpha;
        let rect = RectangleInstance {
            color: [0.0, 0.0, 0.0, alpha],
            size: [window_w, window_h],
            position: [0.0, 0.0],
        };
        self.transition_pipeline
            .update_instance_buffer(&mut gpu.encoder, &gpu.device, vec![rect]);
    }

    pub fn render_transition(&self, target: &mut Target, view: &wgpu::TextureView) {
        if self.curr_transition_alpha != 0.0 {
            let mut render_pass =
                target
                    .gpu
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });
            self.transition_pipeline
                .render(&target.transform_uniform, &mut render_pass);
        }
    }
}

impl SceneTransition {
    pub fn scene_type(&self) -> SceneType {
        match &self.mode {
            TransitionMode::Static(scene) => scene.scene_type(),
            _ => SceneType::Transition,
        }
    }
    pub fn resize(&mut self, target: &mut Target) {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.resize(target),
            TransitionMode::FadeIn(scene) => scene.resize(target),
            TransitionMode::FadeOut(from, _to) => {
                from.resize(target);
                // to.resize(target);
            }
            _ => {}
        }
    }
    pub fn update(&mut self, target: &mut Target) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.update(target),
            TransitionMode::FadeIn(scene) => {
                scene.update(target);

                let mut alpha = 1.0 - self.n;

                self.n += 0.05;
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
                self.set_transition_alpha(&mut target.gpu, alpha, window_w, window_h);
                SceneEvent::None
            }
            TransitionMode::FadeOut(from, _to) => {
                from.update(target);

                let alpha = 0.0 + self.n;

                self.n += 0.05;
                if self.n >= 1.0 {
                    self.n = 0.0;
                    self.active = false;

                    let next = std::mem::replace(&mut self.mode, TransitionMode::None);

                    let game_scene = if let TransitionMode::FadeOut(from, to) = next {
                        from.done(target);
                        to(target)
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
                self.set_transition_alpha(&mut target.gpu, alpha, window_w, window_h);
                SceneEvent::None
            }
            _ => SceneEvent::None,
        }
    }
    pub fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        match &mut self.mode {
            TransitionMode::FadeIn(scene) => scene.render(target, view),
            TransitionMode::FadeOut(from, _to) => from.render(target, view),
            TransitionMode::Static(scene) => scene.render(target, view),
            _ => {}
        }

        self.render_transition(target, view);
    }
    pub fn window_event(&mut self, target: &mut Target, event: &WindowEvent) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::Static(scene) => scene.window_event(target, event),
            _ => SceneEvent::None,
        }
    }
    pub fn main_events_cleared(&mut self, target: &mut Target) -> SceneEvent {
        match &mut self.mode {
            TransitionMode::FadeIn(scene) => scene.main_events_cleared(target),
            TransitionMode::FadeOut(from, _to) => from.main_events_cleared(target),
            TransitionMode::Static(scene) => scene.main_events_cleared(target),
            _ => SceneEvent::None,
        }
    }
}
