mod iced_menu;

mod icons;

use std::time::Duration;

use iced_menu::AppUi;
use iced_runtime::task::BoxFuture;
use neothesia_core::render::BgPipeline;

use winit::event::WindowEvent;

use crate::{
    context::Context,
    iced_utils::{
        iced_conversion,
        iced_state::{self, Program},
    },
    scene::Scene,
    song::Song,
};

use std::task::Waker;

type Renderer = iced_wgpu::Renderer;

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    iced_state: iced_state::State<AppUi>,

    context: std::task::Context<'static>,
    futures: Vec<BoxFuture<iced_menu::Message>>,
}

impl MenuScene {
    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let menu = AppUi::new(ctx, song);
        let iced_state =
            iced_state::State::new(menu, ctx.iced_manager.viewport.logical_size(), ctx);

        Self {
            bg_pipeline: BgPipeline::new(&ctx.gpu),
            iced_state,

            context: std::task::Context::from_waker(noop_waker_ref()),
            futures: Vec::new(),
        }
    }
}

impl Scene for MenuScene {
    #[profiling::function]
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.bg_pipeline.update_time(delta);
        self.iced_state.tick(ctx);

        self.futures
            .retain_mut(|f| match f.as_mut().poll(&mut self.context) {
                std::task::Poll::Ready(msg) => {
                    self.iced_state.queue_message(msg);
                    false
                }
                std::task::Poll::Pending => true,
            });

        if let Some(tasks) = self.iced_state.update(ctx) {
            for fut in tasks.into_iter().flat_map(|task| task.into_future()) {
                self.futures.push(fut);
            }
        }
    }

    #[profiling::function]
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.bg_pipeline.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        use winit::keyboard::ModifiersState;

        let modifiers = ModifiersState::default();

        if let Some(event) = iced_conversion::window_event(
            event.clone(),
            ctx.iced_manager.viewport.scale_factor(),
            modifiers,
        ) {
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

fn noop_waker_ref() -> &'static Waker {
    use std::ptr::null;
    use std::task::{RawWaker, RawWakerVTable};

    unsafe fn noop_clone(_data: *const ()) -> RawWaker {
        noop_raw_waker()
    }

    unsafe fn noop(_data: *const ()) {}

    const NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

    const fn noop_raw_waker() -> RawWaker {
        RawWaker::new(null(), &NOOP_WAKER_VTABLE)
    }

    struct SyncRawWaker(RawWaker);
    unsafe impl Sync for SyncRawWaker {}

    static NOOP_WAKER_INSTANCE: SyncRawWaker = SyncRawWaker(noop_raw_waker());

    // SAFETY: `Waker` is #[repr(transparent)] over its `RawWaker`.
    unsafe { &*(&NOOP_WAKER_INSTANCE.0 as *const RawWaker as *const Waker) }
}
