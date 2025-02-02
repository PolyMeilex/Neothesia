use std::time::{Duration, Instant};

use lilt::Animated;
use nuon::{
    button::Button, canvas::Canvas, column::Column, container::Container, row::Row, stack::Stack,
    trilayout::TriLayout, Color, Element,
};
use winit::dpi::LogicalSize;

use crate::scene::playing_scene::midi_player::MidiPlayer;

use super::widget::{looper::Looper, progress_bar::ProgressBar, speed_pill::SpeedPill};

#[derive(Debug, Clone)]
pub enum ProgressBarMsg {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub enum LooperMsg {
    MoveStart(Duration),
    MoveEnd(Duration),
    Toggle,
}

#[derive(Clone, Debug)]
pub enum Msg {
    GoBack,
    PauseResume,
    SettingsToggle,

    SpeedDown,
    SpeedUp,

    ProggresBar(ProgressBarMsg),
    Looper(LooperMsg),
}

fn cone_icon() -> &'static str {
    "\u{F2D2}"
}

fn gear_icon() -> &'static str {
    "\u{F3E5}"
}

fn gear_fill_icon() -> &'static str {
    "\u{F3E2}"
}

fn repeat_icon() -> &'static str {
    "\u{f130}"
}

fn play_icon() -> &'static str {
    "\u{f4f4}"
}

fn pause_icon() -> &'static str {
    "\u{f4c3}"
}

fn left_arrow_icon() -> &'static str {
    "\u{f12f}"
}

pub struct UiData<'a> {
    pub is_settings_open: bool,
    pub is_looper_on: bool,
    pub loop_start: Duration,
    pub loop_end: Duration,
    pub speed: f32,

    pub player: &'a MidiPlayer,
    pub window_size: LogicalSize<f32>,

    pub frame_timestamp: Instant,
    pub topbar_expand_animation: &'a Animated<bool, Instant>,
    pub settings_animation: &'a Animated<bool, Instant>,
}

fn header(data: &UiData) -> impl Into<Element<Msg>> {
    Container::new().height(30.0).child(
        TriLayout::new()
            .start(
                Row::new().push(
                    Button::new()
                        .icon(left_arrow_icon())
                        .width(30.0)
                        .on_click(Msg::GoBack),
                ),
            )
            .center(
                SpeedPill::new()
                    .speed(data.speed)
                    .on_minus(Msg::SpeedDown)
                    .on_plus(Msg::SpeedUp),
            )
            .end(
                Row::new()
                    .push(
                        Button::new()
                            .icon(if data.player.is_paused() {
                                play_icon()
                            } else {
                                pause_icon()
                            })
                            .width(30.0)
                            .on_click(Msg::PauseResume),
                    )
                    .push(
                        Button::new()
                            .icon(repeat_icon())
                            .width(30.0)
                            .on_click(Msg::Looper(LooperMsg::Toggle)),
                    )
                    .push(
                        Button::new()
                            .icon(if data.is_settings_open {
                                gear_fill_icon()
                            } else {
                                gear_icon()
                            })
                            .width(30.0)
                            .on_click(Msg::SettingsToggle),
                    ),
            ),
    )
}

#[profiling::function]
pub fn top_bar(data: UiData) -> impl Into<Element<Msg>> {
    let header = header(&data);

    let timeline = Container::new().height(45.0).child(
        Row::new().push(
            Stack::new()
                .push(
                    ProgressBar::new()
                        .color(Color::new_u8(56, 145, 255, 1.0))
                        .on_press(Msg::ProggresBar(ProgressBarMsg::Pressed))
                        .on_release(Msg::ProggresBar(ProgressBarMsg::Released)),
                )
                .when(data.is_looper_on, |stack| {
                    stack.push(
                        Looper::new()
                            .start(data.loop_start)
                            .end(data.loop_end)
                            .on_start_move(|p| Msg::Looper(LooperMsg::MoveStart(p)))
                            .on_end_move(|p| Msg::Looper(LooperMsg::MoveEnd(p))),
                    )
                }),
        ),
    );

    let body = Column::new().push(header).push(timeline);

    let y = data
        .topbar_expand_animation
        .animate_bool(-75.0 + 5.0, 0.0, data.frame_timestamp);

    Stack::new()
        .push(
            Container::new()
                .y(y)
                .background(Color::new_u8(37, 35, 42, 1.0))
                .child(body),
        )
        .push({
            let card_w = 300.0;
            let card_x = data
                .settings_animation
                .animate_bool(card_w, 0.0, data.frame_timestamp);
            let card_x = card_x + data.window_size.width - card_w;

            Container::new()
                .y(y + 30.0 + 45.0)
                .x(card_x)
                .height(100.0)
                .width(card_w)
                .background(Color::new_u8(37, 35, 42, 1.0))
                .border_radius([10.0, 0.0, 10.0, 0.0])
                .child(Canvas::new(|renderer, layout| {
                    let x = layout.x;
                    let y = layout.y;
                    let w = layout.w;
                    let size = 50.0;
                    let half_size = size / 2.0;

                    renderer.icon(x + w / 2.0 - half_size, y + 10.0, size, cone_icon());
                    renderer.centered_text(x, y + size + 15.0, w, 25.0, 25.0, "WIP");
                }))
        })
}
