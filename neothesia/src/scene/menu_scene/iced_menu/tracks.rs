use iced_core::{Alignment, Length};
use iced_runtime::Command;
use iced_widget::{button, column as col, row, vertical_space};

use crate::{
    iced_utils::iced_state::Element,
    scene::menu_scene::{segment_button, track_card},
    song::PlayerConfig,
    target::Target,
};

use super::{centered_text, theme, Data, Message};

#[derive(Debug, Clone)]
pub enum TracksMessage {
    AllTracksPlayer(PlayerConfig),
    TrackPlayer(usize, PlayerConfig),
    TrackVisibility(usize, bool),
}

impl From<TracksMessage> for Message {
    fn from(msg: TracksMessage) -> Self {
        Message::Tracks(msg)
    }
}

pub(super) fn update(
    _data: &mut Data,
    msg: TracksMessage,
    target: &mut Target,
) -> Command<Message> {
    match msg {
        TracksMessage::AllTracksPlayer(config) => {
            if let Some(song) = target.song.as_mut() {
                for track in song.config.tracks.iter_mut() {
                    track.player = config.clone();
                }
            }
        }
        TracksMessage::TrackPlayer(track, config) => {
            if let Some(song) = target.song.as_mut() {
                song.config.tracks[track].player = config;
            }
        }
        TracksMessage::TrackVisibility(track, visible) => {
            if let Some(song) = target.song.as_mut() {
                song.config.tracks[track].visible = visible;
            }
        }
    }

    Command::none()
}

pub(super) fn view<'a>(_data: &'a Data, target: &Target) -> Element<'a, Message> {
    let mut tracks = Vec::new();
    if let Some(song) = target.song.as_ref() {
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
                let color_id = track.track_color_id % target.config.color_schema.len();
                let color = &target.config.color_schema[color_id].base;
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

            let body = segment_button::segment_button()
                .button(
                    "Mute",
                    TracksMessage::TrackPlayer(track.track_id, PlayerConfig::Mute).into(),
                )
                .button(
                    "Auto",
                    TracksMessage::TrackPlayer(track.track_id, PlayerConfig::Auto).into(),
                )
                .button(
                    "Human",
                    TracksMessage::TrackPlayer(track.track_id, PlayerConfig::Human).into(),
                )
                .active(active)
                .active_color(color)
                .build();

            let card = track_card::track_card()
                .title(name)
                .subtitle(format!("{} Notes", track.notes.len()))
                .track_color(color)
                .body(body);

            let card = if track.has_drums && !track.has_other_than_drums {
                card
            } else {
                card.on_icon_press(TracksMessage::TrackVisibility(track.track_id, !visible).into())
            };

            tracks.push(card.build().into());
        }
    }

    let controls = {
        let listen = button(centered_text("Listen Only"))
            .on_press(TracksMessage::AllTracksPlayer(PlayerConfig::Auto).into())
            .style(theme::button());

        let play_along = button(centered_text("Play Along"))
            .on_press(TracksMessage::AllTracksPlayer(PlayerConfig::Human).into())
            .style(theme::button());

        row![listen, play_along].spacing(14)
    };

    let column = super::super::wrap::Wrap::with_elements(tracks)
        .spacing(14.0)
        .line_spacing(14.0)
        .padding(50.0)
        .align_items(Alignment::Center);

    let column = col![vertical_space(Length::Fixed(30.0)), controls, column]
        .align_items(Alignment::Center)
        .width(Length::Fill);

    iced_widget::scrollable(column).into()
}
