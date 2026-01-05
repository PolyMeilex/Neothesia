mod state;
use bytes::Bytes;
use state::{Page, UiState};

mod midi_picker;
use midi_picker::open_midi_file_picker;

mod neo_btn;
use neo_btn::{neo_btn, neo_btn_icon};

mod settings;
mod tracks;

use std::{future::Future, time::Duration};

use crate::utils::{BoxFuture, window::WinitEvent};
use neothesia_core::render::{BgPipeline, ImageIdentifier, QuadRenderer, TextRenderer};

use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use crate::{NeothesiaEvent, context::Context, icons, scene::Scene, song::Song};

use std::task::Waker;

use super::NuonRenderer;

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

    logo: ImageIdentifier,

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
        let logo = nuon_renderer.add_image(neothesia_core::render::Image::new(
            &ctx.gpu.device,
            &ctx.gpu.queue,
            logo,
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

                if neo_btn().size(btn_w, btn_h).label("No").build(ui) {
                    self.state.go_back();
                }

                nuon::translate().x(btn_w).add_to_current(ui);
                nuon::translate().x(btn_gap).add_to_current(ui);

                if neo_btn().size(btn_w, btn_h).label("Yes").build(ui) {
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
                nuon::image(self.logo)
                    .x(-logo_w / 2.0)
                    .size(logo_w, logo_h)
                    .build(ui);

                nuon::translate()
                    .x(-w / 2.0)
                    .y(logo_h + post_logo_gap)
                    .build(ui, |ui| {
                        if neo_btn().size(w, h).label("Select File").build(ui) {
                            self.futures.push(open_midi_file_picker(&mut self.state));
                        }

                        nuon::translate().y(h + gap).add_to_current(ui);

                        if neo_btn().size(w, h).label("Settings").build(ui) {
                            self.state.go_to(Page::Settings);
                        }

                        nuon::translate().y(h + gap).add_to_current(ui);

                        if neo_btn().size(w, h).label("Exit").build(ui) {
                            self.state.go_back();
                        }
                    });
            });

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            let gap = 10.0;
            let btn_w = 80.0;
            let btn_h = 60.0;

            nuon::translate().y(-gap).add_to_current(ui);
            nuon::translate().y(-btn_h).add_to_current(ui);

            if let Some(song) = self.state.song() {
                nuon::label()
                    .text(&song.file.name)
                    .size(win_w, 60.0)
                    .font_size(16.0)
                    .build(ui);
            }

            nuon::translate().build(ui, |ui| {
                nuon::translate().x(gap).add_to_current(ui);

                if neo_btn()
                    .size(btn_w, btn_h)
                    .icon(icons::cone_icon())
                    .color([100; 3])
                    .tooltip("FreePlay")
                    .build(ui)
                {
                    state::freeplay(&self.state, ctx);
                }
            });

            if self.state.song().is_none() {
                return;
            }

            nuon::translate().x(win_w).build(ui, |ui| {
                nuon::translate().x(-btn_w - gap).add_to_current(ui);

                if neo_btn()
                    .size(btn_w, btn_h)
                    .icon(icons::play_icon())
                    .tooltip("Play")
                    .build(ui)
                {
                    state::play(&self.state, ctx);
                }

                nuon::translate().x(-btn_w - gap).add_to_current(ui);

                if neo_btn()
                    .size(btn_w, btn_h)
                    .icon(icons::note_list_icon())
                    .tooltip("Tracks")
                    .build(ui)
                {
                    self.state.go_to(Page::TrackSelection);
                }
            });
        });
    }
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

        if event.cursor_moved() {
            self.nuon.mouse_move(
                ctx.window_state.cursor_logical_position.x,
                ctx.window_state.cursor_logical_position.y,
            );
        } else if event.left_mouse_pressed() {
            self.nuon.mouse_down();
        } else if event.left_mouse_released() {
            self.nuon.mouse_up();
        } else if event.back_mouse_pressed() {
            self.state.go_back();
        }

        match self.state.current() {
            Page::Exit => {
                if event.key_pressed(Key::Named(NamedKey::Enter)) {
                    ctx.proxy.send_event(NeothesiaEvent::Exit).unwrap();
                }

                if event.key_pressed(Key::Named(NamedKey::Escape)) {
                    self.state.go_back();
                }
            }
            Page::Main => {
                if event.key_pressed(Key::Named(NamedKey::Tab)) {
                    self.futures.push(open_midi_file_picker(&mut self.state));
                }

                if event.key_pressed(Key::Named(NamedKey::Enter)) {
                    state::play(&self.state, ctx)
                }

                if event.key_pressed(Key::Named(NamedKey::Escape)) {
                    self.state.go_back();
                }

                if event.key_pressed(Key::Character("s")) {
                    self.state.go_to(Page::Settings);
                }

                if event.key_pressed(Key::Character("t")) {
                    self.state.go_to(Page::TrackSelection);
                }

                if event.key_pressed(Key::Character("f")) {
                    state::freeplay(&self.state, ctx);
                }
            }
            Page::Settings => {
                if event.key_pressed(Key::Named(NamedKey::Escape)) {
                    self.state.go_back();
                }
            }
            Page::TrackSelection => {
                if event.key_pressed(Key::Named(NamedKey::Enter)) {
                    state::play(&self.state, ctx);
                }

                if event.key_pressed(Key::Named(NamedKey::Escape)) {
                    self.state.go_back();
                }
            }
        }
    }
}

fn noop_waker_ref() -> &'static Waker {
    use std::{
        ptr::null,
        task::{RawWaker, RawWakerVTable},
    };

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
