use std::time::{Duration, Instant};

use crate::{NeothesiaEvent, context::Context, icons, locals};

use super::{
    PlayingScene,
    animation::{Animated, Easing},
};

pub struct TopBar {
    pub topbar_expand_animation: Animated<bool, Instant>,
    is_expanded: bool,

    settings_animation: Animated<bool, Instant>,
    /// Number of lyrics lines currently shown, animated so the bar smoothly grows/shrinks
    lyrics_lines_animation: Animated<f32, Instant>,

    settings_active: bool,

    looper_active: bool,
    loop_start: Duration,
    loop_end: Duration,
}

impl TopBar {
    pub fn new() -> Self {
        Self {
            topbar_expand_animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),
            settings_animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),
            lyrics_lines_animation: Animated::new(1.0)
                .duration(500.)
                .easing(Easing::EaseOutExpo),

            is_expanded: false,
            settings_active: false,

            looper_active: false,
            loop_start: Duration::ZERO,
            loop_end: Duration::ZERO,
        }
    }

    pub fn is_looper_active(&self) -> bool {
        self.looper_active
    }

    pub fn loop_start_timestamp(&self) -> Duration {
        self.loop_start
    }

    pub fn loop_end_timestamp(&self) -> Duration {
        self.loop_end
    }

    #[profiling::function]
    pub fn update(scene: &mut PlayingScene, ctx: &mut Context) {
        let PlayingScene { top_bar, .. } = scene;

        let window_state = &ctx.window_state;

        let h = 75.0;
        let is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        top_bar.is_expanded = is_hovered;
        top_bar.is_expanded |= top_bar.settings_active;

        top_bar
            .topbar_expand_animation
            .transition(top_bar.is_expanded, ctx.frame_timestamp);
        top_bar
            .settings_animation
            .transition(top_bar.settings_active, ctx.frame_timestamp);

        Self::ui(scene, ctx);
    }

    #[profiling::function]
    pub fn ui(this: &mut PlayingScene, ctx: &mut Context) {
        let mut ui = std::mem::replace(&mut this.nuon, nuon::Ui::new());

        // POC For sheet music symbol rendering
        if false {
            nuon::translate().y(100.0).build(&mut ui, |ui| {
                let spacing = 15.0;
                let half_spacing = spacing / 2.0;
                let y = 50.0;
                let x = 40.0;

                let color = [0; 3];

                nuon::quad()
                    .color([255; 3])
                    .pos(0.0, 0.0)
                    .size(250.0, 130.0)
                    .border_radius([0.0, 10.0, 10.0, 0.0])
                    .build(ui);

                // Staves
                for id in 0..5 {
                    nuon::quad()
                        .y(y - id as f32 * spacing + 39.0)
                        .height(1.0)
                        .width(250.0)
                        .color(color)
                        .build(ui);
                }

                let base = || half_spacing * 4.0;
                let pitch_y = |pitch: u8| base() - half_spacing * pitch as f32;

                nuon::leland()
                    .text(icons::sheet::g_clef())
                    .color(color)
                    .x(x)
                    .y(y)
                    .build(ui);

                let whole = icons::sheet::notehead_whole();

                for pitch in 0..10 {
                    let y = y + pitch_y(pitch);

                    nuon::leland()
                        .text(whole)
                        .color(color)
                        .x(x + 60.0 + 22.0 * (pitch % 2) as f32)
                        .y(y)
                        .build(ui);
                }
            });
        }

        let bar_w = ctx.window_state.logical_size.width;
        let marker_x = bar_w / 3.0;

        // Variable-speed strip: syllables are packed with a uniform gap, and the scroll
        // speed changes so each syllable crosses the marker exactly at its timestamp
        let offset = this.lyrics.strip_offset(
            this.player.time_without_lead_in(),
            ctx.config.animation_speed(),
        );
        let is_visible = |sylable: &super::lyrics::Lyric| {
            let x = marker_x + sylable.x - offset;
            x + sylable.width >= 0.0 && x <= bar_w
        };

        // Only make room for as many lines as the on-screen syllables need
        let visible_lines = this
            .lyrics
            .lyrics
            .iter()
            .filter(|s| is_visible(s))
            .map(|s| s.line + 1)
            .max()
            .unwrap_or(1);
        this.top_bar
            .lyrics_lines_animation
            .transition(visible_lines as f32, ctx.frame_timestamp);

        nuon::translate().y(100.0).build(&mut ui, |ui| {
            let line_h = this.lyrics.font_size * 1.25;
            let lines = this
                .top_bar
                .lyrics_lines_animation
                .animate(|v| v, ctx.frame_timestamp);
            let bar_h = this.lyrics.font_size + 40.0 + (lines - 1.0) * line_h;

            nuon::quad()
                .size(bar_w, bar_h)
                .color([0, 0, 0, 250])
                .build(ui);
            nuon::quad()
                .size(bar_w, 1.0)
                .color([255, 255, 255, 10])
                .build(ui);
            nuon::quad()
                .size(bar_w, 1.0)
                .y(bar_h)
                .color([255, 255, 255, 10])
                .build(ui);

            // #1 Time based placement
            for sylable in this.lyrics.lyrics.iter() {
                let r = 10.0;
                let x = marker_x
                    + ((sylable.timestamp.as_secs_f32() + this.player.leed_in().as_secs_f32())
                        - this.player.time().as_secs_f32())
                        * ctx.config.animation_speed();

                // Already hit the marker
                if x < marker_x {
                    continue;
                }

                // Centered on `x`, so the circle's center crosses the marker on time
                nuon::circle(r)
                    .x(x - r)
                    .color([255, 255, 255, 10])
                    .build(ui);
            }

            nuon::quad()
                .x(marker_x)
                .size(2.0, bar_h)
                .color([255, 255, 255, 60])
                .build(ui);

            for sylable in this.lyrics.lyrics.iter() {
                let x = marker_x + sylable.x - offset;

                if x + sylable.width < 0.0 {
                    continue;
                }
                if x > bar_w {
                    break;
                }

                let color = if sylable.x <= offset {
                    [255, 255, 255, 120]
                } else {
                    [255, 255, 255, 255]
                };

                nuon::label()
                    .text(&sylable.text)
                    .font_size(this.lyrics.font_size)
                    .color(color)
                    .text_justify(nuon::TextAlign::Start)
                    .text_align(nuon::TextAlign::Center)
                    .x(x)
                    .y(20.0 + sylable.line as f32 * line_h)
                    .size(f32::MAX, this.lyrics.font_size)
                    .build(ui);
            }
        });

        nuon::translate()
            .y(this.top_bar.topbar_expand_animation.animate_bool(
                -75.0 + 5.0,
                0.0,
                ctx.frame_timestamp,
            ))
            .build(&mut ui, |ui| {
                Self::panel(this, ctx, ui);
            });

        this.nuon = ui;
    }

    fn panel(this: &mut PlayingScene, ctx: &mut Context, ui: &mut nuon::Ui) {
        let win_w = ctx.window_state.logical_size.width;

        nuon::quad()
            .size(win_w, 30.0 + 45.0)
            .color([37, 35, 42])
            .build(ui);

        Self::panel_left(this, ctx, ui);
        Self::panel_center(this, ctx, ui);
        Self::panel_right(this, ctx, ui);
        Self::settings_panel(this, ctx, ui);

        // ProggressBar
        nuon::translate().y(30.0).build(ui, |ui| {
            Self::proggress_bar(this, ctx, ui);
        });
    }

    fn button() -> nuon::Button {
        nuon::button().size(30.0, 30.0).border_radius([5.0; 4])
    }

    fn panel_left(this: &mut PlayingScene, ctx: &mut Context, ui: &mut nuon::Ui) {
        if Self::button().icon(icons::left_arrow_icon()).build(ui) {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(Some(this.player.song().clone())))
                .ok();
        }
    }

    fn panel_center(_this: &mut PlayingScene, ctx: &mut Context, ui: &mut nuon::Ui) {
        let win_w = ctx.window_state.logical_size.width;
        let pill_w = 45.0 * 2.0;

        nuon::translate()
            .x(win_w / 2.0 - pill_w / 2.0)
            .y(5.0)
            .build(ui, |ui| {
                if nuon::button()
                    .size(45.0, 20.0)
                    .color([67, 67, 67])
                    .hover_color([87, 87, 87])
                    .preseed_color([97, 97, 97])
                    .border_radius([10.0, 0.0, 0.0, 10.0])
                    .icon(icons::minus_icon())
                    .text_justify(nuon::TextAlign::Start)
                    .build(ui)
                {
                    ctx.config
                        .set_speed_multiplier(ctx.config.speed_multiplier() - 0.1);
                }

                nuon::label()
                    .text(format!(
                        "{}%",
                        (ctx.config.speed_multiplier() * 100.0).round()
                    ))
                    .bold(true)
                    .size(45.0 * 2.0, 20.0)
                    .build(ui);

                if nuon::button()
                    .size(45.0, 20.0)
                    .x(45.0)
                    .color([67, 67, 67])
                    .hover_color([87, 87, 87])
                    .preseed_color([97, 97, 97])
                    .border_radius([0.0, 10.0, 10.0, 0.0])
                    .icon(icons::plus_icon())
                    .text_justify(nuon::TextAlign::End)
                    .build(ui)
                {
                    ctx.config
                        .set_speed_multiplier(ctx.config.speed_multiplier() + 0.1);
                }
            });
    }

    fn panel_right(this: &mut PlayingScene, ctx: &mut Context, ui: &mut nuon::Ui) {
        nuon::translate()
            .x(ctx.window_state.logical_size.width)
            .build(ui, |ui| {
                nuon::translate().x(-30.0).add_to_current(ui);

                if Self::button()
                    .icon(if this.top_bar.settings_active {
                        icons::gear_fill_icon()
                    } else {
                        icons::gear_icon()
                    })
                    .build(ui)
                {
                    this.top_bar.settings_active = !this.top_bar.settings_active;
                }

                nuon::translate().x(-30.0).add_to_current(ui);

                if Self::button().icon(icons::repeat_icon()).build(ui) {
                    this.top_bar.looper_active = !this.top_bar.looper_active;

                    // Looper enabled for the first time
                    if this.top_bar.looper_active
                        && this.top_bar.loop_start.is_zero()
                        && this.top_bar.loop_end.is_zero()
                    {
                        this.top_bar.loop_start = this.player.time();
                        this.top_bar.loop_end = this.player.time() + Duration::from_secs(5);
                    }
                }

                nuon::translate().x(-30.0).add_to_current(ui);

                if Self::button()
                    .icon(if this.player.is_paused() {
                        icons::play_icon()
                    } else {
                        icons::pause_icon()
                    })
                    .build(ui)
                {
                    this.player.pause_resume();
                }
            });
    }

    fn settings_panel(this: &mut PlayingScene, ctx: &mut Context, ui: &mut nuon::Ui) {
        let width = 280.0;
        let offset = this
            .top_bar
            .settings_animation
            .animate_bool(0.0, width, ctx.frame_timestamp);

        nuon::translate()
            .x(ctx.window_state.logical_size.width - offset)
            .y(75.0)
            .build(ui, |ui| {
                // Gap
                nuon::translate().y(5.0).add_to_current(ui);

                nuon::quad()
                    .size(width, 100.0)
                    .color([37, 35, 42])
                    .border_radius([10.0, 0.0, 0.0, 10.0])
                    .build(ui);

                nuon::translate().x(15.0).build(ui, |ui| {
                    nuon::settings_section(locals::playing::display())
                        .width(width - 30.0)
                        .build(ui, |ui, rows, _| {
                            if nuon::settings_row_toggler()
                                .title(locals::playing::chord_identifier())
                                .subtitle(locals::playing::chord_identifier_subtitle())
                                .value(ctx.config.chord_identifier())
                                .build(ui, rows)
                            {
                                ctx.config
                                    .set_chord_identifier(!ctx.config.chord_identifier());
                            }
                        });
                });
            });
    }

    fn proggress_bar(this: &mut PlayingScene, ctx: &mut Context, ui: &mut nuon::Ui) {
        let h = 45.0;
        let w = ctx.window_state.logical_size.width;

        let render_looper = Self::proggress_bar_looper(this, ctx, ui, w, h);

        Self::proggress_bar_bg(this, ctx, ui, w, h);

        render_looper(ui);
    }

    fn proggress_bar_bg(
        this: &mut PlayingScene,
        ctx: &mut Context,
        ui: &mut nuon::Ui,
        w: f32,
        h: f32,
    ) {
        let progress_w = w * this.player.percentage();

        match nuon::click_area("ProggressBar").size(w, h).build(ui) {
            nuon::ClickAreaEvent::PressStart { .. } => {
                if !this.rewind_controller.is_rewinding() {
                    this.rewind_controller.start_mouse_rewind(&mut this.player);

                    let x = ctx.window_state.cursor_logical_position.x;
                    let w = ctx.window_state.logical_size.width;

                    let p = x / w;
                    this.player.set_percentage_time(p);
                    this.keyboard.reset_notes();
                }
            }
            nuon::ClickAreaEvent::PressEnd { .. } => {
                this.rewind_controller.stop_rewind(&mut this.player);
            }
            nuon::ClickAreaEvent::Idle { .. } => {}
        }

        nuon::quad()
            .size(progress_w, h)
            .color([56, 145, 255])
            .build(ui);

        for m in this.player.song().file.measures.iter() {
            let length = this.player.length().as_secs_f32();
            let start = this.player.leed_in().as_secs_f32() / length;
            let measure = m.as_secs_f32() / length;

            let x = (start + measure) * w;

            let light_measure = nuon::Color::new(1.0, 1.0, 1.0, 0.5);
            let dark_measure = nuon::Color::new(0.4, 0.4, 0.4, 1.0);

            let color = if x < progress_w {
                light_measure
            } else {
                dark_measure
            };

            nuon::quad().x(x).size(1.0, h).color(color).build(ui);
        }
    }

    fn proggress_bar_looper<'a>(
        this: &mut PlayingScene,
        ctx: &mut Context,
        ui: &mut nuon::Ui,
        w: f32,
        h: f32,
    ) -> impl FnOnce(&mut nuon::Ui) + 'a {
        let loop_start = this.top_bar.loop_start;
        let loop_start = this.player.time_to_percentage(&loop_start) * w;

        let loop_end = this.top_bar.loop_end;
        let loop_end = this.player.time_to_percentage(&loop_end) * w;

        let loop_h = h + 10.0;

        let looper_active = this.top_bar.looper_active;

        let (loop_start_ev, loop_end_ev) = if looper_active {
            let loop_start_ev = nuon::click_area("LooperStart")
                .x(loop_start)
                .width(5.0)
                .height(loop_h)
                .build(ui);
            let loop_end_ev = nuon::click_area("LooperEnd")
                .x(loop_end)
                .width(5.0)
                .height(loop_h)
                .build(ui);
            (loop_start_ev, loop_end_ev)
        } else {
            (nuon::ClickAreaEvent::null(), nuon::ClickAreaEvent::null())
        };

        if loop_start_ev.is_pressed() {
            let x = ctx.window_state.cursor_logical_position.x;
            let w = ctx.window_state.logical_size.width;
            let p = x / w;

            if p * w < loop_end - 10.0 {
                this.top_bar.loop_start = this.player.percentage_to_time(p);
            }
        }

        if loop_end_ev.is_pressed() {
            let x = ctx.window_state.cursor_logical_position.x;
            let w = ctx.window_state.logical_size.width;
            let p = x / w;

            if p * w > loop_start + 10.0 {
                this.top_bar.loop_end = this.player.percentage_to_time(p);
            }
        }

        // render
        move |ui| {
            if !looper_active {
                return;
            }

            let color = [255, 56, 187];
            let white = [255; 3];

            nuon::quad()
                .x(loop_start)
                .width(loop_end - loop_start)
                .height(loop_h)
                .color([255, 56, 187, 90])
                .build(ui);

            nuon::quad()
                .x(loop_start)
                .width(5.0)
                .height(loop_h)
                .color(
                    if loop_start_ev.is_hovered() || loop_start_ev.is_pressed() {
                        white
                    } else {
                        color
                    },
                )
                .build(ui);

            nuon::quad()
                .x(loop_end)
                .width(5.0)
                .height(loop_h)
                .color(if loop_end_ev.is_hovered() || loop_end_ev.is_pressed() {
                    white
                } else {
                    color
                })
                .build(ui);
        }
    }
}
