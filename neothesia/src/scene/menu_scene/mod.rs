mod state;
use bytes::Bytes;
use state::{Page, UiState};

mod midi_picker;
use midi_picker::open_midi_file_picker;

mod settings;
mod tracks;

use std::{future::Future, time::Duration};

use crate::utils::BoxFuture;
use neothesia_core::render::{BgPipeline, QuadRenderer, TextRenderer};

use winit::{
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

use crate::{context::Context, scene::Scene, song::Song, NeothesiaEvent};

use std::task::Waker;

use super::NuonRenderer;

mod icons {
    pub fn play_icon() -> &'static str {
        "\u{f4f4}"
    }

    pub fn note_list_icon() -> &'static str {
        "\u{f49f}"
    }

    pub fn left_arrow_icon() -> &'static str {
        "\u{f12f}"
    }

    pub fn caret_down() -> &'static str {
        "\u{f229}"
    }
}

type MsgFn = Box<dyn FnOnce(&mut UiState, &mut Context)>;

fn on_async<T, Fut, FN>(future: Fut, f: FN) -> BoxFuture<MsgFn>
where
    T: 'static,
    Fut: Future<Output = T> + Send + 'static,
    FN: FnOnce(T, &mut UiState, &mut Context) + Send + 'static,
{
    Box::pin(async {
        let res = future.await;
        let f: MsgFn = Box::new(move |data, ctx| f(res, data, ctx));
        f
    })
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
enum Popup {
    #[default]
    None,
    OutputSelector,
    InputSelector,
}

impl Popup {
    fn toggle(&mut self, new: Self) {
        *self = if *self == new { Self::None } else { new }
    }

    fn close(&mut self) {
        *self = Self::None;
    }
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    text_renderer: TextRenderer,
    nuon_renderer: NuonRenderer,

    logo: Bytes,

    state: UiState,

    context: std::task::Context<'static>,
    futures: Vec<BoxFuture<MsgFn>>,

    quad_pipeline: QuadRenderer,
    nuon: nuon::Ui,

    tracks_scroll: nuon::ScrollState,
    settings_scroll: nuon::ScrollState,
    popup: Popup,
}

impl MenuScene {
    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let iced_state = UiState::new(ctx, song);

        let quad_pipeline = ctx.quad_renderer_facotry.new_renderer();
        let text_renderer = ctx.text_renderer_factory.new_renderer();

        let mut nuon_renderer = NuonRenderer::new(ctx);

        let logo = Bytes::from_static(include_bytes!("../../../../assets/banner.png"));
        nuon_renderer.add_image(neothesia_core::render::Image::new(
            &ctx.gpu.device,
            &ctx.gpu.queue,
            logo.clone(),
        ));

        Self {
            bg_pipeline: BgPipeline::new(&ctx.gpu),
            text_renderer,
            state: iced_state,
            nuon_renderer,

            logo,

            context: std::task::Context::from_waker(noop_waker_ref()),
            futures: Vec::new(),

            quad_pipeline,
            nuon: nuon::Ui::new(),
            tracks_scroll: nuon::ScrollState::new(),
            settings_scroll: nuon::ScrollState::new(),
            popup: Popup::None,
        }
    }

    fn main_ui(&mut self, ctx: &mut Context) {
        if self.state.is_loading() {
            let width = ctx.window_state.logical_size.width;
            let height = ctx.window_state.logical_size.height;

            nuon::label()
                .size(width, height)
                .font_size(30.0)
                .text("Loading...")
                .text_justify(nuon::TextJustify::Center)
                .build(&mut self.nuon);
            return;
        }

        let mut nuon = std::mem::replace(&mut self.nuon, nuon::Ui::new());

        match self.state.current() {
            Page::Exit => self.exit_page_ui(ctx, &mut nuon),
            Page::Main => self.main_page_ui(ctx, &mut nuon),
            Page::Settings => self.settings_page_ui(ctx, &mut nuon),
            Page::TrackSelection => self.tracks_page_ui(ctx, &mut nuon),
        }

        self.nuon = nuon;
    }

    fn exit_page_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;

        let btn_w = 320.0;
        let btn_h = 50.0;
        let btn_gap = 5.0;

        let text_h = 80.0;

        let full_w = btn_w * 2.0 + btn_gap;
        let full_h = btn_h + text_h;

        nuon::translate()
            .x(nuon::center_x(win_w, full_w))
            .y(nuon::center_y(win_h, full_h))
            .build(ui, |ui| {
                nuon::label()
                    .text("Do you want to exit?")
                    .font_size(30.0)
                    .size(full_w, text_h)
                    .build(ui);

                nuon::translate().y(text_h).add_to_current(ui);

                if neo_btn(ui, btn_w, btn_h, "No") {
                    self.state.go_back();
                }

                nuon::translate().x(btn_w).add_to_current(ui);
                nuon::translate().x(btn_gap).add_to_current(ui);

                if neo_btn(ui, btn_w, btn_h, "Yes") {
                    ctx.proxy.send_event(NeothesiaEvent::Exit).ok();
                }
            });
    }

    fn main_page_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;

        let w = 450.0;
        let h = 80.0;
        let gap = 10.0;

        let logo_w = 650.0;
        let logo_h = 118.0;
        let post_logo_gap = 40.0;

        nuon::translate()
            .x(win_w / 2.0)
            .y(win_h / 5.0)
            .build(ui, |ui| {
                nuon::image(self.logo.clone())
                    .x(-logo_w / 2.0)
                    .size(logo_w, logo_h)
                    .build(ui);

                nuon::translate()
                    .x(-w / 2.0)
                    .y(logo_h + post_logo_gap)
                    .build(ui, |ui| {
                        if neo_btn(ui, w, h, "Select File") {
                            self.futures.push(open_midi_file_picker(&mut self.state));
                        }

                        nuon::translate().y(h + gap).add_to_current(ui);

                        if neo_btn(ui, w, h, "Settings") {
                            self.state.go_to(Page::Settings);
                        }

                        nuon::translate().y(h + gap).add_to_current(ui);

                        if neo_btn(ui, w, h, "Exit") {
                            self.state.go_back();
                        }
                    });
            });

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            let Some(song) = self.state.song() else {
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
                    state::play(&self.state, ctx);
                }

                nuon::translate().x(-btn_w - gap).add_to_current(ui);

                if neo_btn_icon(ui, btn_w, btn_h, icons::note_list_icon()) {
                    self.state.go_to(Page::TrackSelection);
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
        self.state.tick(ctx);

        self.futures
            .retain_mut(|f| match f.as_mut().poll(&mut self.context) {
                std::task::Poll::Ready(msg) => {
                    msg(&mut self.state, ctx);
                    false
                }
                std::task::Poll::Pending => true,
            });

        self.state.tick(ctx);

        self.main_ui(ctx);

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);

        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );
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
        if let WindowEvent::MouseWheel { delta, .. } = event {
            match delta {
                winit::event::MouseScrollDelta::LineDelta(_, y) => {
                    let y = y * 60.0;
                    self.settings_scroll.update(y);
                    self.tracks_scroll.update(y);
                }
                winit::event::MouseScrollDelta::PixelDelta(position) => {
                    self.settings_scroll.update(position.y as f32);
                    self.tracks_scroll.update(position.y as f32);
                }
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

        if let WindowEvent::MouseInput {
            state: ElementState::Pressed,
            button: MouseButton::Back,
            ..
        } = event
        {
            self.state.go_back();
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
            match self.state.current() {
                Page::Exit => {
                    match logical_key {
                        Key::Named(NamedKey::Enter) => {
                            ctx.proxy.send_event(NeothesiaEvent::Exit).unwrap();
                        }
                        Key::Named(NamedKey::Escape) => {
                            self.state.go_back();
                        }
                        _ => {}
                    };
                }
                Page::Main => {
                    match logical_key {
                        Key::Named(key) => match key {
                            NamedKey::Tab => {
                                self.futures.push(open_midi_file_picker(&mut self.state));
                            }
                            NamedKey::Enter => state::play(&self.state, ctx),
                            NamedKey::Escape => {
                                self.state.go_back();
                            }
                            _ => {}
                        },
                        Key::Character(ch) => match ch.as_ref() {
                            "s" => {
                                self.state.go_to(Page::Settings);
                            }
                            "t" => {
                                self.state.go_to(Page::TrackSelection);
                            }
                            _ => {}
                        },
                        _ => {}
                    };
                }
                Page::Settings => {
                    match logical_key {
                        Key::Named(NamedKey::Escape) => {
                            self.state.go_back();
                        }
                        _ => {}
                    };
                }
                Page::TrackSelection => {
                    match logical_key {
                        Key::Named(NamedKey::Enter) => {
                            state::play(&self.state, ctx);
                        }
                        Key::Named(NamedKey::Escape) => {
                            self.state.go_back();
                        }
                        _ => {}
                    };
                }
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
