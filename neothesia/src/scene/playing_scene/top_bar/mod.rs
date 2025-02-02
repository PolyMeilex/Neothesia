use std::time::{Duration, Instant};

use ui::{LooperMsg, ProgressBarMsg};

use crate::{context::Context, NeothesiaEvent};

use super::{
    animation::{Animated, Easing},
    rewind_controller::RewindController,
    PlayingScene,
};

mod renderer;
pub mod ui;
mod widget;

use renderer::NuonRenderer;

pub struct TopBar {
    topbar_expand_animation: Animated<bool, Instant>,
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

    fn on_msg(scene: &mut PlayingScene, ctx: &mut Context, msg: &ui::Msg) {
        use ui::Msg;
        match msg {
            Msg::PauseResume => {
                scene.player.pause_resume();
            }
            Msg::SpeedUp => {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() + 0.1);
            }
            Msg::SpeedDown => {
                ctx.config
                    .set_speed_multiplier(ctx.config.speed_multiplier() - 0.1);
            }
            Msg::SettingsToggle => {
                scene.top_bar.settings_active = !scene.top_bar.settings_active;
            }
            Msg::GoBack => {
                ctx.proxy
                    .send_event(NeothesiaEvent::MainMenu(Some(scene.player.song().clone())))
                    .ok();
            }
            Msg::Looper(msg) => match msg {
                LooperMsg::Toggle => {
                    scene.top_bar.looper_active = !scene.top_bar.looper_active;

                    // Looper enabled for the first time
                    if scene.top_bar.looper_active
                        && scene.top_bar.loop_start.is_zero()
                        && scene.top_bar.loop_end.is_zero()
                    {
                        scene.top_bar.loop_start = scene.player.time();
                        scene.top_bar.loop_end = scene.player.time() + Duration::from_secs(5);
                    }
                }
                LooperMsg::MoveStart(t) => {
                    scene.top_bar.loop_start = *t;
                }
                LooperMsg::MoveEnd(t) => {
                    scene.top_bar.loop_end = *t;
                }
            },
            Msg::ProggresBar(msg) => {
                let PlayingScene {
                    player,
                    keyboard,
                    rewind_controller,
                    ..
                } = scene;

                match msg {
                    ProgressBarMsg::Pressed => {
                        if !rewind_controller.is_rewinding() {
                            rewind_controller.start_mouse_rewind(player);

                            let x = ctx.window_state.cursor_logical_position.x;
                            let w = ctx.window_state.logical_size.width;

                            let p = x / w;
                            player.set_percentage_time(p);
                            keyboard.reset_notes();
                        }
                    }
                    ProgressBarMsg::Released => {
                        if let RewindController::Mouse { .. } = rewind_controller {
                            rewind_controller.stop_rewind(player);
                        }
                    }
                }
            }
        }
    }

    #[profiling::function]
    fn update_nuon(scene: &mut PlayingScene, ctx: &mut Context, _delta: Duration) {
        let globals = nuon::GlobalStore::with(|store| {
            store.insert(&scene.player);
        });

        let mut root = ui::top_bar(ui::UiData {
            window_size: ctx.window_state.logical_size,
            is_settings_open: scene.top_bar.settings_active,
            is_looper_on: scene.top_bar.is_looper_active(),
            speed: ctx.config.speed_multiplier(),
            player: &scene.player,
            loop_start: scene.top_bar.loop_start_timestamp(),
            loop_end: scene.top_bar.loop_end_timestamp(),

            frame_timestamp: ctx.frame_timestamp,
            topbar_expand_animation: &scene.top_bar.topbar_expand_animation,
            settings_animation: &scene.top_bar.settings_animation,
        })
        .into();

        scene.tree.diff(root.as_widget());

        let layout = {
            profiling::scope!("nuon_layout");
            root.as_widget_mut().layout(
                &mut scene.tree,
                &nuon::ParentLayout {
                    x: 0.0,
                    y: 0.0,
                    w: ctx.window_state.logical_size.width,
                    h: ctx.window_state.logical_size.height,
                },
                &nuon::LayoutCtx { globals: &globals },
            )
        };

        let mut messages = vec![];

        scene.nuon_event_queue.dispatch_events(
            &mut messages,
            &mut scene.tree,
            root.as_widget_mut(),
            &layout,
            &globals,
        );

        {
            profiling::scope!("nuon_render");
            root.as_widget().render(
                &mut NuonRenderer {
                    quads: &mut scene.quad_pipeline,
                    text: &mut ctx.text_renderer,
                },
                &layout,
                &scene.tree,
                &nuon::RenderCtx { globals: &globals },
            );
        }

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
        top_bar.is_expanded |= scene.nuon_event_queue.is_mouse_grabbed();

        top_bar
            .topbar_expand_animation
            .transition(top_bar.is_expanded, ctx.frame_timestamp);
        top_bar
            .settings_animation
            .transition(top_bar.settings_active, ctx.frame_timestamp);

        Self::update_nuon(scene, ctx, delta);
    }
}
