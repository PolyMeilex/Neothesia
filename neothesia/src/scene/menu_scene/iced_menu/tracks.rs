use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_widget::{button, column as col, container, row, vertical_space};

use crate::{context::Context, scene::menu_scene::icons, song::PlayerConfig};
use neothesia_iced_widgets::{BarLayout, Element, Layout, NeoBtn};

use super::{
    centered_text,
    page::{Page, PageMessage},
    theme, Data, Message,
};

#[derive(Debug, Clone)]
pub enum Event {
    AllTracksPlayer(PlayerConfig),
    TrackPlayer(usize, PlayerConfig),
    TrackVisibility(usize, bool),
    GoBack,
    Play,
}

pub struct TracksPage;

impl Page for TracksPage {
    type Event = Event;

    fn update(data: &mut Data, event: Event, ctx: &mut Context) -> PageMessage {
        match event {
            Event::AllTracksPlayer(config) => {
                if let Some(song) = ctx.song.as_mut() {
                    for track in song.config.tracks.iter_mut() {
                        track.player = config.clone();
                    }
                }
            }
            Event::TrackPlayer(track, config) => {
                if let Some(song) = ctx.song.as_mut() {
                    song.config.tracks[track].player = config;
                }
            }
            Event::TrackVisibility(track, visible) => {
                if let Some(song) = ctx.song.as_mut() {
                    song.config.tracks[track].visible = visible;
                }
            }
            Event::GoBack => {
                return PageMessage::go_back();
            }
            Event::Play => {
                super::play(data, ctx);
            }
        }

        PageMessage::none()
    }

    fn view<'a>(_data: &'a Data, ctx: &Context) -> Element<'a, Event> {
        let mut tracks = Vec::new();
        if let Some(song) = ctx.song.as_ref() {
            for track in song.file.tracks.iter().filter(|t| !t.notes.is_empty()) {
                let config = &song.config.tracks[track.track_id];

                let visible = config.visible;

                let active = match config.player {
                    PlayerConfig::Mute => 0,
                    PlayerConfig::Auto => 1,
                    PlayerConfig::Human => 2,
                };

                let color = if !visible {
                    iced_core::Color::from_rgb8(102, 102, 102)
                } else {
                    let color_id = track.track_color_id % ctx.config.color_schema.len();
                    let color = &ctx.config.color_schema[color_id].base;
                    iced_core::Color::from_rgb8(color.0, color.1, color.2)
                };

                let name = if track.has_drums && !track.has_other_than_drums {
                    "Percussion"
                } else {
                    let instrument_id = track
                        .programs
                        .last()
                        .map(|p| p.program as usize)
                        .unwrap_or(0);
                    midi_file::INSTRUMENT_NAMES[instrument_id]
                };

                let body = neothesia_iced_widgets::SegmentButton::new()
                    .button(
                        "Mute",
                        Event::TrackPlayer(track.track_id, PlayerConfig::Mute),
                    )
                    .button(
                        "Auto",
                        Event::TrackPlayer(track.track_id, PlayerConfig::Auto),
                    )
                    .button(
                        "Human",
                        Event::TrackPlayer(track.track_id, PlayerConfig::Human),
                    )
                    .active(active)
                    .active_color(color);

                let card = neothesia_iced_widgets::TrackCard::new()
                    .title(name)
                    .subtitle(format!("{} Notes", track.notes.len()))
                    .track_color(color)
                    .body(body);

                let card = if track.has_drums && !track.has_other_than_drums {
                    card
                } else {
                    card.on_icon_press(Event::TrackVisibility(track.track_id, !visible))
                };

                tracks.push(card.into());
            }
        }

        let column = neothesia_iced_widgets::Wrap::with_elements(tracks)
            .spacing(14.0)
            .line_spacing(14.0)
            .padding(30.0)
            .align_items(Alignment::Center);

        let column = col![vertical_space().height(Length::Fixed(30.0)), column]
            .align_items(Alignment::Center)
            .width(Length::Fill);

        let column = iced_widget::scrollable(column);

        let right = {
            let play = NeoBtn::new(
                icons::play_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::Play);

            if ctx.song.is_some() {
                row![play]
            } else {
                row![]
            }
            .spacing(10)
            .width(Length::Shrink)
            .align_items(Alignment::Center)
        };

        let right = container(right)
            .width(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });

        let left = {
            let back = NeoBtn::new(
                icons::left_arrow_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::GoBack);

            row![back].align_items(Alignment::Start)
        };

        let left = container(left)
            .width(Length::Fill)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            });

        let center = {
            let listen = button(centered_text("Listen Only"))
                .on_press(Event::AllTracksPlayer(PlayerConfig::Auto))
                .style(theme::button);

            let play_along = button(centered_text("Play Along"))
                .on_press(Event::AllTracksPlayer(PlayerConfig::Human))
                .style(theme::button);

            row![listen, play_along]
                .width(Length::Shrink)
                .align_items(Alignment::Center)
                .spacing(14)
        };

        let center = container(center)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });

        Layout::new()
            .body(column)
            .bottom(BarLayout::new().left(left).center(center).right(right))
            .into()
    }

    fn keyboard_input(event: &iced_runtime::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Enter => Some(Message::TracksPage(self::Event::Play)),
                Named::Escape => Some(Message::GoBack),
                _ => None,
            },
            _ => None,
        }
    }
}
