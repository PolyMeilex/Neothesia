use std::time::Duration;

use nuon::{
    button::Button, column::Column, container::Container, row::Row, speed_pill::SpeedPill,
    stack::Stack, translate::Translate, trilayout::TriLayout, Color, Element,
};

use crate::scene::playing_scene::midi_player::MidiPlayer;

use super::widget::{looper::Looper, progress_bar::ProgressBar};

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
    pub y: f32,
    pub player: &'a MidiPlayer,
}

#[derive(Default)]
pub struct Header {}

impl Header {
    fn view(&mut self, data: &UiData) -> impl Into<Element<'static, Msg>> {
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
}

#[derive(Default)]
pub struct Ui {
    header: Header,
}

impl Ui {
    pub fn new() -> Self {
        Self::default()
    }

    #[profiling::function]
    pub fn view<'a>(&'a mut self, data: UiData<'a>) -> impl Into<Element<'static, Msg>> {
        let header = self.header.view(&data);

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

        Translate::new().y(data.y).child(
            Container::new()
                .background(Color::new_u8(37, 35, 42, 1.0))
                .child(body),
        )
    }
}
