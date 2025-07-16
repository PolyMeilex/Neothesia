mod iced_menu;

mod midi_picker;
use midi_picker::open_midi_file_picker;

use std::{future::Future, time::Duration};

use iced_core::image::Handle as ImageHandle;
use iced_menu::{AppUi, Data};
use iced_runtime::task::BoxFuture;
use neothesia_core::render::{BgPipeline, QuadRenderer, TextRenderer};

use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::{
    context::Context,
    iced_utils::{
        iced_conversion,
        iced_state::{self, Program},
    },
    scene::Scene,
    song::Song,
    NeothesiaEvent,
};

use std::task::Waker;

use super::NuonRenderer;

mod icons {
    pub fn play_icon() -> &'static str {
        "\u{f4f4}"
    }

    pub fn note_list_icon() -> &'static str {
        "\u{f49f}"
    }

    #[allow(unused)]
    pub fn left_arrow_icon() -> &'static str {
        "\u{f12f}"
    }
}

type Renderer = iced_wgpu::Renderer;

type MsgFn = Box<dyn FnOnce(&mut Data, &mut Context)>;

fn on_async<T, Fut, FN>(future: Fut, f: FN) -> BoxFuture<MsgFn>
where
    T: 'static,
    Fut: Future<Output = T> + Send + 'static,
    FN: FnOnce(T, &mut Data, &mut Context) + Send + 'static,
{
    Box::pin(async {
        let res = future.await;
        let f: MsgFn = Box::new(move |data, ctx| f(res, data, ctx));
        f
    })
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    text_renderer: TextRenderer,
    nuon_renderer: NuonRenderer,

    iced_state: iced_state::State<AppUi>,

    context: std::task::Context<'static>,
    futures: Vec<BoxFuture<iced_menu::Message>>,
    futures2: Vec<BoxFuture<MsgFn>>,

    logo_handle: ImageHandle,

    quad_pipeline: QuadRenderer,
    nuon: nuon::Ui,
}

impl MenuScene {
    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let menu = AppUi::new(ctx, song);
        let iced_state =
            iced_state::State::new(menu, ctx.iced_manager.viewport.logical_size(), ctx);

        let quad_pipeline = ctx.quad_renderer_facotry.new_renderer();
        let text_renderer = ctx.text_renderer_factory.new_renderer();

        Self {
            bg_pipeline: BgPipeline::new(&ctx.gpu),
            text_renderer,
            iced_state,
            nuon_renderer: NuonRenderer::default(),

            context: std::task::Context::from_waker(noop_waker_ref()),
            futures: Vec::new(),
            futures2: Vec::new(),

            logo_handle: ImageHandle::from_bytes(include_bytes!("./img/banner.png").to_vec()),

            quad_pipeline,
            nuon: nuon::Ui::new(),
        }
    }

    fn main_ui(&mut self, ctx: &mut Context) {
        if self.iced_state.program().is_loading() {
            return;
        }

        match self.iced_state.program().current() {
            iced_menu::Step::Exit => self.exit_step_ui(ctx),
            iced_menu::Step::Main => self.main_step_ui(ctx),
            iced_menu::Step::Settings => self.settings_step_ui(ctx),
            iced_menu::Step::TrackSelection => self.tracks_step_ui(ctx),
        }
    }

    fn exit_step_ui(&mut self, ctx: &mut Context) {
        let ui = &mut self.nuon;

        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;

        let btn_w = 320.0;
        let btn_h = 50.0;
        let btn_gap = 5.0;

        let text_h = 80.0;

        let full_w = btn_w * 2.0 + btn_gap;
        let full_h = btn_h + text_h;

        nuon::translate()
            .x(win_w / 2.0)
            .y(win_h / 2.0)
            .build(ui, |ui| {
                nuon::translate()
                    .x(-full_w / 2.0)
                    .y(-full_h / 2.0)
                    .add_to_current(ui);

                nuon::label()
                    .text("Do you want to exit?")
                    .font_size(30.0)
                    .size(full_w, text_h)
                    .build(ui);

                nuon::translate().y(text_h).add_to_current(ui);

                if neo_btn(ui, btn_w, btn_h, "No") {
                    self.iced_state.queue_message(iced_menu::Message::GoBack);
                }

                nuon::translate().x(btn_w).add_to_current(ui);
                nuon::translate().x(btn_gap).add_to_current(ui);

                if neo_btn(ui, btn_w, btn_h, "Yes") {
                    ctx.proxy.send_event(NeothesiaEvent::Exit).ok();
                }
            });
    }

    fn main_step_ui(&mut self, ctx: &mut Context) {
        let ui = &mut self.nuon;

        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;

        let w = 450.0;
        let h = 80.0;
        let gap = 10.0;

        let logo_w = 650.0;
        let logo_h = 115.0;
        let post_logo_gap = 40.0;

        nuon::translate()
            .x(win_w / 2.0)
            .y(win_h / 5.0)
            .build(ui, |ui| {
                nuon::image(self.logo_handle.clone())
                    .x(-logo_w / 2.0)
                    .size(logo_w, logo_h)
                    .build(ui);

                nuon::translate()
                    .x(-w / 2.0)
                    .y(logo_h + post_logo_gap)
                    .build(ui, |ui| {
                        if neo_btn(ui, w, h, "Select File") {
                            self.futures2.push(open_midi_file_picker(
                                &mut self.iced_state.program_mut().data,
                            ));
                        }

                        nuon::translate().y(h + gap).add_to_current(ui);

                        if neo_btn(ui, w, h, "Settings") {
                            self.iced_state.queue_message(iced_menu::Message::GoToPage(
                                iced_menu::Step::Settings,
                            ));
                        }

                        nuon::translate().y(h + gap).add_to_current(ui);

                        if neo_btn(ui, w, h, "Exit") {
                            self.iced_state.queue_message(iced_menu::Message::GoBack);
                        }
                    });
            });

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            let Some(song) = self.iced_state.program().song() else {
                return;
            };

            let gap = 10.0;
            let btn_w = 80.0;
            let btn_h = 60.0;

            nuon::translate().y(-gap).add_to_current(ui);
            nuon::translate().y(-btn_h).add_to_current(ui);

            nuon::label()
                .text(&song.file.name)
                .size(win_w, 60.0)
                .font_size(16.0)
                .build(ui);

            nuon::translate().x(win_w).build(ui, |ui| {
                nuon::translate().x(-btn_w - gap).add_to_current(ui);

                if neo_btn_icon(ui, btn_w, btn_h, icons::play_icon()) {
                    iced_menu::play(&self.iced_state.program().data, ctx);
                }

                nuon::translate().x(-btn_w - gap).add_to_current(ui);

                if neo_btn_icon(ui, btn_w, btn_h, icons::note_list_icon()) {
                    self.iced_state.queue_message(iced_menu::Message::GoToPage(
                        iced_menu::Step::TrackSelection,
                    ));
                }
            });
        });
    }

    fn settings_step_ui(&mut self, ctx: &mut Context) {
        let ui = &mut self.nuon;

        let win_h = ctx.window_state.logical_size.height;

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            // Bottom Margin
            nuon::translate().y(-10.0).add_to_current(ui);

            nuon::translate().y(-60.0).add_to_current(ui);

            let gap = 10.0;
            let w = 80.0;
            let h = 60.0;

            nuon::translate().x(0.0).build(ui, |ui| {
                nuon::translate().x(gap).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::left_arrow_icon()) {
                    self.iced_state.queue_message(iced_menu::Message::GoBack);
                }

                nuon::translate().x(-w - gap).add_to_current(ui);
            });
        });
    }

    fn tracks_step_ui(&mut self, ctx: &mut Context) {
        let ui = &mut self.nuon;

        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            // Bottom Margin
            nuon::translate().y(-10.0).add_to_current(ui);

            nuon::translate().y(-60.0).add_to_current(ui);

            let gap = 10.0;
            let w = 80.0;
            let h = 60.0;

            nuon::translate().x(0.0).build(ui, |ui| {
                nuon::translate().x(gap).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::left_arrow_icon()) {
                    self.iced_state.queue_message(iced_menu::Message::GoBack);
                }

                nuon::translate().x(-w - gap).add_to_current(ui);
            });

            nuon::translate().x(win_w).build(ui, |ui| {
                nuon::translate().x(-w - gap).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::play_icon()) {
                    iced_menu::play(&self.iced_state.program().data, ctx);
                }
            });
        });
    }
}

fn neo_btn(ui: &mut nuon::Ui, w: f32, h: f32, label: &str) -> bool {
    neo_btn_child(ui, label, w, h, |ui| {
        nuon::label()
            .text(label)
            .size(w, h)
            .font_size(30.0)
            .build(ui);
    })
}

fn neo_btn_icon(ui: &mut nuon::Ui, w: f32, h: f32, icon: &str) -> bool {
    neo_btn_child(ui, icon, w, h, |ui| {
        nuon::label()
            .icon(icon)
            .size(w, h)
            .font_size(30.0)
            .build(ui);
    })
}

fn neo_btn_child(
    ui: &mut nuon::Ui,
    id: impl Into<nuon::Id>,
    w: f32,
    h: f32,
    child: impl FnOnce(&mut nuon::Ui),
) -> bool {
    let event = nuon::click_area(id).size(w, h).build(ui);

    let (bg, accent) = if event.is_hovered() || event.is_pressed() {
        (
            nuon::Color::new_u8(9, 9, 9, 0.6),
            nuon::Color::new_u8(56, 145, 255, 1.0),
        )
    } else {
        (
            nuon::Color::new_u8(17, 17, 17, 0.6),
            nuon::Color::new_u8(160, 81, 255, 1.0),
        )
    };

    nuon::quad()
        .size(w, h)
        .color(bg)
        .border_radius([7.0; 4])
        .build(ui);
    nuon::quad()
        .size(w, 7.0)
        .y(h - 7.0)
        .color(accent)
        .border_radius([0.0, 0.0, 7.0, 7.0])
        .build(ui);

    child(ui);

    event.is_clicked()
}

impl Scene for MenuScene {
    #[profiling::function]
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.quad_pipeline.clear();
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

        self.futures2
            .retain_mut(|f| match f.as_mut().poll(&mut self.context) {
                std::task::Poll::Ready(msg) => {
                    msg(&mut self.iced_state.program_mut().data, ctx);
                    false
                }
                std::task::Poll::Pending => true,
            });

        if let Some(tasks) = self.iced_state.update(ctx) {
            for fut in tasks.into_iter().flat_map(|task| task.into_future()) {
                self.futures.push(fut);
            }
        }

        self.main_ui(ctx);

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);

        self.text_renderer
            .update(ctx.window_state.logical_size.into());
        self.quad_pipeline.prepare();
    }

    #[profiling::function]
    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.bg_pipeline.render(rpass);
        self.quad_pipeline.render(rpass);
        self.text_renderer.render(rpass);
        self.nuon_renderer.render(rpass);
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

        if let WindowEvent::CursorMoved { .. } = event {
            self.nuon.mouse_move(
                ctx.window_state.cursor_logical_position.x,
                ctx.window_state.cursor_logical_position.y,
            );
        }

        if let WindowEvent::MouseInput {
            state,
            button: MouseButton::Left,
            ..
        } = event
        {
            match state {
                ElementState::Pressed => self.nuon.mouse_down(),
                ElementState::Released => self.nuon.mouse_up(),
            }
        }

        if let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    state: ElementState::Pressed,
                    logical_key,
                    ..
                },
            ..
        } = event
        {
            match self.iced_state.program().current() {
                iced_menu::Step::Exit => {
                    match logical_key {
                        Key::Named(NamedKey::Enter) => {
                            ctx.proxy.send_event(NeothesiaEvent::Exit).unwrap();
                        }
                        Key::Named(NamedKey::Escape) => {
                            self.iced_state.queue_message(iced_menu::Message::GoBack);
                        }
                        _ => {}
                    };
                }
                iced_menu::Step::Main => {
                    use iced_menu::{Message, Step};

                    let mut queue_message = |msg| {
                        self.iced_state.queue_message(msg);
                    };

                    match logical_key {
                        Key::Named(key) => match key {
                            NamedKey::Tab => {
                                self.futures2.push(open_midi_file_picker(
                                    &mut self.iced_state.program_mut().data,
                                ));
                            }
                            NamedKey::Enter => {
                                iced_menu::play(&self.iced_state.program().data, ctx)
                            }
                            NamedKey::Escape => {
                                queue_message(Message::GoBack);
                            }
                            _ => {}
                        },
                        Key::Character(ch) => match ch.as_ref() {
                            "s" => {
                                queue_message(Message::GoToPage(Step::Settings));
                            }
                            "t" => {
                                queue_message(Message::GoToPage(Step::TrackSelection));
                            }
                            _ => {}
                        },
                        _ => {}
                    };
                }
                iced_menu::Step::Settings => {}
                iced_menu::Step::TrackSelection => {}
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
