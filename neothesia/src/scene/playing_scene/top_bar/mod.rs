use std::time::{Duration, Instant};

use ui::LooperMsg;

use crate::context::Context;

use super::{
    animation::{Animated, Easing},
    PlayingScene,
};

mod renderer;
pub mod ui;
mod widget;

use renderer::NuonRenderer;

pub struct TopBar {
    pub topbar_expand_animation: Animated<bool, Instant>,
    is_expanded: bool,

    settings_animation: Animated<bool, Instant>,

    pub settings_active: bool,

    pub looper_active: bool,
    pub loop_start: Duration,
    pub loop_end: Duration,
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

    fn on_msg(scene: &mut PlayingScene, _ctx: &mut Context, msg: &ui::Msg) {
        use ui::Msg;
        match msg {
            Msg::Looper(msg) => match msg {
                LooperMsg::MoveStart(t) => {
                    scene.top_bar.loop_start = *t;
                }
                LooperMsg::MoveEnd(t) => {
                    scene.top_bar.loop_end = *t;
                }
            },
        }
    }

    #[profiling::function]
    fn update_nuon(scene: &mut PlayingScene, ctx: &mut Context, _delta: Duration) {
        let globals = nuon::GlobalStore::with(|store| {
            store.insert(&scene.player);
        });

        let mut root = ui::top_bar(ui::UiData {
            window_size: ctx.window_state.logical_size,
            is_looper_on: scene.top_bar.is_looper_active(),
            loop_start: scene.top_bar.loop_start_timestamp(),
            loop_end: scene.top_bar.loop_end_timestamp(),

            frame_timestamp: ctx.frame_timestamp,
            topbar_expand_animation: &scene.top_bar.topbar_expand_animation,
            settings_animation: &scene.top_bar.settings_animation,
        })
        .into();

        let messages = scene.nuon.update(
            root.as_widget_mut(),
            &globals,
            ctx.window_state.logical_size.width,
            ctx.window_state.logical_size.height,
            &mut NuonRenderer {
                quads: &mut scene.quad_pipeline,
                text: &mut ctx.text_renderer,
            },
        );

        drop(root);

        for msg in messages.iter() {
            Self::on_msg(scene, ctx, msg);
        }
    }

    #[profiling::function]
    pub fn update(scene: &mut PlayingScene, ctx: &mut Context, delta: Duration) {
        let PlayingScene { top_bar, .. } = scene;

        let window_state = &ctx.window_state;

        let h = 75.0;
        let is_hovered = window_state.cursor_logical_position.y < h * 1.7;

        top_bar.is_expanded = is_hovered;
        top_bar.is_expanded |= top_bar.settings_active;
        top_bar.is_expanded |= scene.nuon.event_queue.is_mouse_grabbed();

        top_bar
            .topbar_expand_animation
            .transition(top_bar.is_expanded, ctx.frame_timestamp);
        top_bar
            .settings_animation
            .transition(top_bar.settings_active, ctx.frame_timestamp);

        Self::update_nuon(scene, ctx, delta);
    }
}
