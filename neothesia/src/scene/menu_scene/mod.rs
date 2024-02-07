mod iced_menu;

mod icons;
mod layout;
mod neo_btn;
mod piano_range;
mod preferences_group;
mod scroll_listener;
mod segment_button;
mod track_card;
mod wrap;

use std::time::Duration;

use iced_menu::AppUi;
use neothesia_core::render::BgPipeline;

use wgpu_jumpstart::{TransformUniform, Uniform};
use winit::event::WindowEvent;

use crate::{
    iced_utils::{
        iced_conversion,
        iced_state::{self, Program},
    },
    scene::Scene,
    target::Target,
};

type Renderer = iced_wgpu::Renderer;

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    iced_state: iced_state::State<AppUi>,

    context: std::task::Context<'static>,
    futures: Vec<futures::future::BoxFuture<'static, iced_menu::Message>>,
}

impl MenuScene {
    pub fn new(target: &mut Target) -> Self {
        let menu = AppUi::new(target);
        let iced_state =
            iced_state::State::new(menu, target.iced_manager.viewport.logical_size(), target);

        let mut scene = Self {
            bg_pipeline: BgPipeline::new(&target.gpu),
            iced_state,

            context: std::task::Context::from_waker(futures::task::noop_waker_ref()),
            futures: Vec::new(),
        };

        scene.resize(target);
        scene
    }
}

impl Scene for MenuScene {
    fn update(&mut self, target: &mut Target, delta: Duration) {
        self.bg_pipeline.update_time(&mut target.gpu, delta);
        self.iced_state.queue_message(iced_menu::Message::Tick);

        self.futures
            .retain_mut(|f| match f.as_mut().poll(&mut self.context) {
                std::task::Poll::Ready(msg) => {
                    self.iced_state.queue_message(msg);
                    false
                }
                std::task::Poll::Pending => true,
            });

        if !self.iced_state.is_queue_empty() {
            if let Some(command) = self.iced_state.update(target) {
                for a in command.actions() {
                    match a {
                        iced_runtime::command::Action::Future(f) => {
                            self.futures.push(f);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn render<'pass>(
        &'pass mut self,
        _transform: &'pass Uniform<TransformUniform>,
        rpass: &mut wgpu::RenderPass<'pass>,
    ) {
        self.bg_pipeline.render(rpass);
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        use winit::keyboard::ModifiersState;

        let modifiers = ModifiersState::default();

        if let Some(event) = iced_conversion::window_event(
            event.clone(),
            target.iced_manager.viewport.scale_factor(),
            modifiers,
        ) {
            self.iced_state.queue_event(event.clone());

            match &event {
                iced_core::event::Event::Mouse(event) => {
                    if let Some(msg) = self.iced_state.program().mouse_input(event, target) {
                        self.iced_state.queue_message(msg);
                    }
                }
                iced_core::event::Event::Keyboard(event) => {
                    if let Some(msg) = self.iced_state.program().keyboard_input(event, target) {
                        self.iced_state.queue_message(msg);
                    }
                }
                _ => {}
            }
        }
    }
}
