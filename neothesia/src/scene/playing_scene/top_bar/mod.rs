use std::time::{Duration, Instant};

use crate::{NeothesiaEvent, context::Context, icons};

use super::{
    PlayingScene,
    animation::{Animated, Easing},
};

pub struct TopBar {
    pub topbar_expand_animation: Animated<bool, Instant>,
    is_expanded: bool,

    settings_animation: Animated<bool, Instant>,

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
                    .text_justify(nuon::TextJustify::Left)
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
                    .text_justify(nuon::TextJustify::Right)
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
            nuon::ClickAreaEvent::PressStart => {
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
