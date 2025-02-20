mod iced_menu;

mod icons;

use std::time::Duration;

use iced_menu::AppUi;
use iced_runtime::Action;
use neothesia_core::render::{BgPipeline, QuadPipeline};

use wgpu_jumpstart::{TransformUniform, Uniform};
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

use super::playing_scene::NuonRenderer;

type Renderer = iced_wgpu::Renderer;

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    iced_state: iced_state::State<AppUi>,

    context: std::task::Context<'static>,
    futures: Vec<futures::stream::BoxStream<'static, Action<iced_menu::Message>>>,

    quad_pipeline: QuadPipeline,
    nuon: nuon::State,
}

impl MenuScene {
    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let menu = AppUi::new(ctx, song);
        let iced_state =
            iced_state::State::new(menu, ctx.iced_manager.viewport.logical_size(), ctx);

        let mut quad_pipeline = QuadPipeline::new(&ctx.gpu, &ctx.transform);
        quad_pipeline.init_layer(&ctx.gpu, 500);
        quad_pipeline.init_layer(&ctx.gpu, 500);

        Self {
            bg_pipeline: BgPipeline::new(&ctx.gpu),
            iced_state,

            context: std::task::Context::from_waker(futures::task::noop_waker_ref()),
            futures: Vec::new(),

            quad_pipeline,
            nuon: nuon::State::new(),
        }
    }
}

impl Scene for MenuScene {
    #[profiling::function]
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.bg_pipeline.update_time(&mut ctx.gpu, delta);
        self.iced_state.tick(ctx);

        self.futures
            .retain_mut(|f| match f.as_mut().poll_next(&mut self.context) {
                std::task::Poll::Ready(a) => match a {
                    Some(Action::Output(msg)) => {
                        self.iced_state.queue_message(msg);
                        true
                    }
                    Some(_) => true,
                    None => false,
                },
                std::task::Poll::Pending => true,
            });

        // Let's skip for now, as we need the tick update every frame anyway
        // if self.iced_state.is_queue_empty() {
        //     return;
        // }

        if let Some(task) = self.iced_state.update(ctx) {
            if let Some(fut) = iced_runtime::task::into_stream(task) {
                self.futures.push(fut);
            }
        }

        self.quad_pipeline.clear();
        {
            #[derive(Debug, Clone)]
            enum Msg {
                Click,
            }

            let globals = nuon::GlobalStore::with(|store| {});

            let mut root = nuon::trilayout::TriLayout::new().center(
                nuon::container::Container::new()
                    .y(ctx.window_state.logical_size.height * 0.25 + 100.0)
                    .width(450.0)
                    .child(
                        nuon::column::Column::new()
                            .gap(10.0)
                            .push(
                                nuon::neo_button::NeoButton::new()
                                    .label("Select File")
                                    .width(450.0)
                                    .height(80.0)
                                    .on_click(iced_menu::Message::MainPage(
                                        iced_menu::main::Event::MidiFilePicker(
                                            iced_menu::main::MidiFilePickerMessage::open(),
                                        ),
                                    )),
                            )
                            .push(
                                nuon::neo_button::NeoButton::new()
                                    .label("Settings")
                                    .width(450.0)
                                    .height(80.0)
                                    .on_click(iced_menu::Message::GoToPage(
                                        iced_menu::Step::Settings,
                                    )),
                            )
                            .push(
                                nuon::neo_button::NeoButton::new()
                                    .label("Exit")
                                    .width(450.0)
                                    .height(80.0)
                                    .on_click(iced_menu::Message::GoBack),
                            ),
                    ),
            );

            let messages = self.nuon.update(
                &mut root,
                &globals,
                ctx.window_state.logical_size.width,
                ctx.window_state.logical_size.height,
                &mut NuonRenderer {
                    quads: &mut self.quad_pipeline,
                    text: &mut ctx.text_renderer,
                },
            );

            // dbg!(&messages);

            for msg in messages {
                self.iced_state.queue_message(msg);
            }
        }

        self.quad_pipeline.prepare(&ctx.gpu.device, &ctx.gpu.queue);
    }

    #[profiling::function]
    fn render<'pass>(
        &'pass mut self,
        transform: &'pass Uniform<TransformUniform>,
        rpass: &mut wgpu::RenderPass<'pass>,
    ) {
        self.bg_pipeline.render(rpass);
        self.quad_pipeline.render(0, transform, rpass);
        self.quad_pipeline.render(1, transform, rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        self.nuon
            .event_queue
            .push_winit_event(event, ctx.window_state.scale_factor);

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
