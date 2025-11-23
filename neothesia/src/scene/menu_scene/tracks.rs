use midi_file::MidiTrack;
use nuon::TextJustify;
use std::hash::Hash;

use crate::{
    context::Context,
    song::{PlayerConfig, TrackConfig},
};

use super::{icons, neo_btn_icon, state};

pub const CARD_W: f32 = 344.0;
pub const CARD_H: f32 = 126.0;

impl super::MenuScene {
    pub fn tracks_page_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;
        let bottom_bar_h = 60.0;

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            // Bottom Margin
            nuon::translate().y(-10.0).add_to_current(ui);

            nuon::translate().y(-bottom_bar_h).add_to_current(ui);

            let gap = 10.0;
            let w = 80.0;
            let h = bottom_bar_h;

            nuon::translate().x(0.0).build(ui, |ui| {
                nuon::translate().x(gap).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::left_arrow_icon()) {
                    self.state.go_back();
                }

                nuon::translate().x(-w - gap).add_to_current(ui);
            });

            nuon::translate().x(win_w).build(ui, |ui| {
                nuon::translate().x(-w - gap).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::play_icon()) {
                    state::play(&self.state, ctx);
                }
            });
        });

        if let Some(song) = self.state.song.as_mut() {
            self.tracks_scroll = nuon::scroll()
                .scissor_size(win_w, (win_h - bottom_bar_h).max(0.0))
                .scroll(self.tracks_scroll)
                .build(ui, |ui| {
                    let gap = 14.0;

                    let mut tracks = song
                        .file
                        .tracks
                        .iter()
                        .filter(|t| !t.notes.is_empty())
                        .enumerate();

                    let layout = CardsLayout::new(win_w, tracks.clone().count());

                    let top_margin = 60.0;

                    nuon::translate().y(top_margin).add_to_current(ui);

                    loop {
                        let mut end = false;
                        nuon::translate()
                            .x(nuon::center_x(win_w, layout.width))
                            .build(ui, |ui| {
                                for _ in 0..layout.columns {
                                    let Some((id, track)) = tracks.next() else {
                                        end = true;
                                        break;
                                    };

                                    let config = &song.config.tracks[track.track_id];

                                    match self::track_card(
                                        ctx,
                                        ui,
                                        nuon::Id::hash_with(|h| {
                                            "track_card".hash(h);
                                            id.hash(h);
                                        }),
                                        track,
                                        config,
                                    ) {
                                        TrackCardEvent::PlayerConfig(player) => {
                                            song.config.tracks[track.track_id].player = player;
                                        }
                                        TrackCardEvent::SetVisible(visible) => {
                                            song.config.tracks[track.track_id].visible = visible;
                                        }
                                        TrackCardEvent::Idle => {}
                                    }

                                    nuon::translate().x(CARD_W + gap).add_to_current(ui);
                                }
                            });

                        nuon::translate().y(CARD_H + gap).add_to_current(ui);

                        if end {
                            break;
                        }
                    }
                });
        }
    }
}

struct CardsLayout {
    columns: u8,
    width: f32,
}

impl CardsLayout {
    fn new(w: f32, tracks_count: usize) -> Self {
        const GAP: f32 = 14.0;

        const LAYOUT_1: f32 = CARD_W;
        const LAYOUT_2: f32 = LAYOUT_1 + GAP + CARD_W;
        const LAYOUT_3: f32 = LAYOUT_2 + GAP + CARD_W;

        let columns = if w > LAYOUT_3 {
            3
        } else if w > LAYOUT_2 {
            2
        } else {
            1
        };

        let columns = columns.min(tracks_count).max(1) as u8;

        Self {
            columns,
            width: match columns {
                3 => LAYOUT_3,
                2 => LAYOUT_2,
                _ => LAYOUT_1,
            },
        }
    }
}

#[derive(Debug)]
enum TrackCardEvent {
    PlayerConfig(PlayerConfig),
    SetVisible(bool),
    Idle,
}

fn track_card(
    ctx: &Context,
    ui: &mut nuon::Ui,
    id: impl Into<nuon::Id>,
    track: &MidiTrack,
    config: &TrackConfig,
) -> TrackCardEvent {
    let card_w = CARD_W;
    let card_h = CARD_H;

    let pad = 16.0;

    let id = id.into();

    let track_color = if !config.visible {
        nuon::Color::new_u8(102, 102, 102, 1.0)
    } else {
        let color_id = track.track_color_id % ctx.config.color_schema().len();
        let color = &ctx.config.color_schema()[color_id].base;
        nuon::Color::new_u8(color.0, color.1, color.2, 1.0)
    };

    let title = if track.has_drums && !track.has_other_than_drums {
        "Percussion"
    } else {
        let instrument_id = track
            .programs
            .last()
            .map(|p| p.program as usize)
            .unwrap_or(0);
        midi_file::INSTRUMENT_NAMES[instrument_id]
    };

    let subtitle = format!("{} Notes", track.notes.len());

    nuon::quad()
        .size(card_w, card_h)
        .color([37, 35, 42])
        .border_radius([12.0; 4])
        .build(ui);

    let inner_card_w = card_w - pad * 2.0;
    let _inner_card_h = card_h - pad * 2.0;

    let icon_size = 40.0;

    let mut res = TrackCardEvent::Idle;

    nuon::translate().pos(pad, pad).build(ui, |ui| {
        let accent = track_color;
        let accent_hover = nuon::Color::new(
            (accent.r + 0.05).min(1.0),
            (accent.g + 0.05).min(1.0),
            (accent.b + 0.05).min(1.0),
            1.0,
        );

        let regular = nuon::Color::from([74, 68, 88]);
        let regular_hover = nuon::Color::from([87, 81, 101]);

        if nuon::button()
            .id(nuon::Id::hash_with(|h| {
                id.as_raw().hash(h);
                "visible".hash(h);
            }))
            .size(icon_size, icon_size)
            .color(accent)
            .hover_color(accent_hover)
            .preseed_color(accent)
            .border_radius([255.0; 4])
            .build(ui)
        {
            res = TrackCardEvent::SetVisible(!config.visible);
        }

        let btn_w = inner_card_w / 3.0;

        let labels_x = icon_size + 15.0;
        nuon::translate().x(labels_x).build(ui, |ui| {
            let label_h = icon_size / 2.0;
            let label_w = inner_card_w - labels_x;

            nuon::label()
                .size(label_w, label_h)
                .text(title)
                .text_justify(TextJustify::Left)
                .font_size(16.0)
                .build(ui);

            nuon::label()
                .y(label_h)
                .size(label_w, label_h)
                .text(subtitle)
                .text_justify(TextJustify::Left)
                .font_size(14.0)
                .build(ui);
        });

        nuon::translate().y(icon_size + 15.0).build(ui, |ui| {
            let color = |m: PlayerConfig| {
                if m == config.player { accent } else { regular }
            };
            let hover_color = |m: PlayerConfig| {
                if m == config.player {
                    accent_hover
                } else {
                    regular_hover
                }
            };

            if nuon::button()
                .id(nuon::Id::hash_with(|h| {
                    id.as_raw().hash(h);
                    "mute".hash(h);
                }))
                .x(0.0)
                .size(btn_w, 40.0)
                .color(color(PlayerConfig::Mute))
                .hover_color(hover_color(PlayerConfig::Mute))
                .preseed_color(color(PlayerConfig::Mute))
                .border_radius([255.0, 0.0, 0.0, 255.0])
                .label("Mute")
                .build(ui)
            {
                res = TrackCardEvent::PlayerConfig(PlayerConfig::Mute);
            }

            if nuon::button()
                .id(nuon::Id::hash_with(|h| {
                    id.as_raw().hash(h);
                    "auto".hash(h);
                }))
                .x(btn_w)
                .size(btn_w, 40.0)
                .color(color(PlayerConfig::Auto))
                .hover_color(hover_color(PlayerConfig::Auto))
                .preseed_color(color(PlayerConfig::Auto))
                .border_radius([0.0; 4])
                .label("Auto")
                .build(ui)
            {
                res = TrackCardEvent::PlayerConfig(PlayerConfig::Auto);
            }

            if nuon::button()
                .id(nuon::Id::hash_with(|h| {
                    id.as_raw().hash(h);
                    "human".hash(h);
                }))
                .x(btn_w * 2.0)
                .size(btn_w, 40.0)
                .color(color(PlayerConfig::Human))
                .hover_color(hover_color(PlayerConfig::Human))
                .preseed_color(color(PlayerConfig::Human))
                .border_radius([0.0, 255.0, 255.0, 0.0])
                .label("Human")
                .build(ui)
            {
                res = TrackCardEvent::PlayerConfig(PlayerConfig::Human);
            }
        });
    });

    res
}
