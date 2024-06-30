use std::time::{Duration, Instant};

use neothesia_core::render::TextRenderer;
use wgpu_jumpstart::Color;

use crate::{context::Context, scene::playing_scene::PlayingScene};

use super::{draw_rect, Msg, LOOPER, WHITE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopTickSide {
    Start,
    End,
}

#[derive(Debug, Clone)]
pub enum LoopTickMsg {
    DragStart,
    Drag,
    DragEnd,
}

#[derive(Debug, Clone)]
pub enum LooperMsg {
    Tick {
        side: LoopTickSide,
        event: LoopTickMsg,
    },
    Toggle,
}

struct LoopTick {
    id: nuon::ElementId,
    timestamp: Duration,
}

impl LoopTick {
    fn new(elements: &mut nuon::ElementsMap<Msg>, side: LoopTickSide) -> Self {
        let id = elements.insert(
            nuon::ElementBuilder::new()
                .name(match side {
                    LoopTickSide::Start => "LoopStartTick",
                    LoopTickSide::End => "LoopEndTick",
                })
                .on_click(Msg::LooperEvent(LooperMsg::Tick {
                    side,
                    event: LoopTickMsg::DragStart,
                }))
                .on_cursor_move(Msg::LooperEvent(LooperMsg::Tick {
                    side,
                    event: LoopTickMsg::Drag,
                }))
                .on_release(Msg::LooperEvent(LooperMsg::Tick {
                    side,
                    event: LoopTickMsg::DragEnd,
                })),
        );

        Self {
            id,
            timestamp: Duration::ZERO,
        }
    }
}

pub struct Looper {
    start_tick: LoopTick,
    end_tick: LoopTick,
    is_active: bool,
}

impl Looper {
    pub(super) fn new(elements: &mut nuon::ElementsMap<Msg>) -> Self {
        let start_tick = LoopTick::new(elements, LoopTickSide::Start);
        let end_tick = LoopTick::new(elements, LoopTickSide::End);
        Self {
            start_tick,
            end_tick,
            is_active: false,
        }
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn start_timestamp(&self) -> Duration {
        self.start_tick.timestamp
    }

    pub fn end_timestamp(&self) -> Duration {
        self.end_tick.timestamp
    }

    pub fn on_msg(scene: &mut PlayingScene, ctx: &mut Context, msg: LooperMsg) {
        let PlayingScene {
            top_bar,
            ref player,
            ..
        } = scene;
        let looper = &mut top_bar.looper;
        let elements = &mut top_bar.elements;

        match msg {
            LooperMsg::Tick { side, event } => {
                let tick = match side {
                    LoopTickSide::Start => &mut looper.start_tick,
                    LoopTickSide::End => &mut looper.end_tick,
                };

                match event {
                    LoopTickMsg::DragStart => {
                        elements.set_mouse_grab(Some(tick.id));
                    }
                    LoopTickMsg::Drag => {
                        let w = ctx.window_state.logical_size.width;
                        let x = ctx.window_state.cursor_logical_position.x;
                        tick.timestamp = scene.player.percentage_to_time(x / w);
                    }
                    LoopTickMsg::DragEnd => {
                        elements.set_mouse_grab(None);
                    }
                }
            }
            LooperMsg::Toggle => {
                looper.is_active = !looper.is_active;
                if looper.is_active {
                    looper.start_tick.timestamp = player.time();
                    looper.end_tick.timestamp = looper.start_timestamp() + Duration::from_secs(3);
                }
            }
        }
    }

    pub fn update(scene: &mut PlayingScene, _text: &mut TextRenderer, now: &Instant) {
        let PlayingScene {
            top_bar,
            quad_pipeline,
            ref player,
            ..
        } = scene;
        let looper = &top_bar.looper;
        let elements = &mut top_bar.elements;

        if !looper.is_active {
            elements.update(looper.start_tick.id, nuon::Rect::zero());
            elements.update(looper.end_tick.id, nuon::Rect::zero());
            return;
        }

        let y = top_bar
            .animation
            .animate(-top_bar.bbox.size.height - 30.0, 30.0, *now);
        let alpha = top_bar.animation.animate(0.0, 1.0, *now);

        let h = top_bar.loop_tick_height;
        let w = top_bar.bbox.size.width;

        let tick_size = nuon::Size::new(5.0, h);
        let tick_pos = |start: &Duration| nuon::Point::new(player.time_to_percentage(start) * w, y);

        let loop_start_tick_rect = nuon::Rect::new(tick_pos(&looper.start_timestamp()), tick_size);
        let loop_end_tick_rect = nuon::Rect::new(tick_pos(&looper.end_timestamp()), tick_size);
        elements.update(looper.start_tick.id, loop_start_tick_rect);
        elements.update(looper.end_tick.id, loop_end_tick_rect);

        let start_hovered = elements
            .get(looper.start_tick.id)
            .map(|e| e.hovered())
            .unwrap_or(false)
            || elements.current_mouse_grab_id() == Some(looper.start_tick.id);
        let end_hovered = elements
            .get(looper.end_tick.id)
            .map(|e| e.hovered())
            .unwrap_or(false)
            || elements.current_mouse_grab_id() == Some(looper.end_tick.id);

        let mut start_color = if start_hovered { WHITE } else { LOOPER };
        let mut end_color = if end_hovered { WHITE } else { LOOPER };

        start_color.a *= alpha;
        end_color.a *= alpha;

        let color = Color {
            a: 0.35 * alpha,
            ..LOOPER
        };

        let length = loop_end_tick_rect.origin.x - loop_start_tick_rect.origin.x;

        let mut bg_box = loop_start_tick_rect;
        bg_box.size.width = length;

        draw_rect(quad_pipeline, &bg_box, &color);
        draw_rect(quad_pipeline, &loop_start_tick_rect, &start_color);
        draw_rect(quad_pipeline, &loop_end_tick_rect, &end_color);
    }
}
