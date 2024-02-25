mod iced_menu;

mod icons;

use std::{cell::RefCell, rc::Rc, time::Duration};

use iced_menu::AppUi;
use neothesia_core::render::BgPipeline;

use wgpu_jumpstart::{TransformUniform, Uniform};
use winit::event::WindowEvent;

use crate::{
    context::Context,
    iced_utils::{
        iced_conversion,
        iced_state::{self, Program},
        IcedManager,
    },
    scene::Scene,
};

type Renderer = iced_wgpu::Renderer;

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    iced_state: iced_state::State<AppUi>,

    iced_manager: Rc<RefCell<IcedManager>>,
    context: std::task::Context<'static>,
    futures: Vec<futures::future::BoxFuture<'static, iced_menu::Message>>,
}

impl MenuScene {
    pub fn new(ctx: &mut Context, iced_manager: Rc<RefCell<IcedManager>>) -> Self {
        let menu = AppUi::new(ctx);
        let bounds = iced_manager.borrow().viewport.logical_size();
        let iced_state =
            iced_state::State::new(menu, bounds, ctx, &mut iced_manager.borrow_mut().renderer);

        Self {
            bg_pipeline: BgPipeline::new(&ctx.gpu),
            iced_state,
            iced_manager,

            context: std::task::Context::from_waker(futures::task::noop_waker_ref()),
            futures: Vec::new(),
        }
    }
}

impl Scene for MenuScene {
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.bg_pipeline.update_time(&mut ctx.gpu, delta);
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
            if let Some(command) = self
                .iced_state
                .update(ctx, &mut self.iced_manager.borrow_mut())
            {
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

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        use winit::keyboard::ModifiersState;

        let modifiers = ModifiersState::default();

        if let Some(event) =
            iced_conversion::window_event(event.clone(), ctx.window_state.scale_factor, modifiers)
        {
            self.iced_state.queue_event(event.clone());

            match &event {
                iced_core::event::Event::Mouse(event) => {
                    if let Some(msg) = self.iced_state.program().mouse_input(event, ctx) {
                        self.iced_state.queue_message(msg);
                    }
                }
                iced_core::event::Event::Keyboard(event) => {
                    if let Some(msg) = self.iced_state.program().keyboard_input(event, ctx) {
                        self.iced_state.queue_message(msg);
                    }
                }
                _ => {}
            }
        }
    }
}
