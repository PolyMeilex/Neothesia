use std::path::PathBuf;

use iced_runtime::Task;
use iced_widget::row;
use neothesia_iced_widgets::Element;

use crate::context::Context;

use super::{
    page::{Page, PageMessage},
    Data, Message,
};

#[derive(Debug, Clone)]
pub enum RangeUpdateKind {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum Event {
    VerticalGuidelines(bool),
    HorizontalGuidelines(bool),
    Glow(bool),
    NoteLabels(bool),
    SeparateChannels(bool),
    OpenSoundFontPicker,
    SoundFontFileLoaded(Option<PathBuf>),

    RangeStart(RangeUpdateKind),
    RangeEnd(RangeUpdateKind),
    AudioGain(RangeUpdateKind),
}

pub struct SettingsPage;

impl SettingsPage {
    pub fn open_sound_font_picker() -> Event {
        Event::OpenSoundFontPicker
    }
}

impl Page for SettingsPage {
    type Event = Event;

    fn update(data: &mut Data, msg: Event, ctx: &mut Context) -> PageMessage {
        match msg {
            Event::VerticalGuidelines(v) => {
                ctx.config.set_vertical_guidelines(v);
            }
            Event::HorizontalGuidelines(v) => {
                ctx.config.set_horizontal_guidelines(v);
            }
            Event::Glow(v) => {
                ctx.config.set_glow(v);
            }
            Event::NoteLabels(v) => {
                ctx.config.set_note_labels(v);
            }
            Event::SeparateChannels(v) => {
                ctx.config.set_separate_channels(v);
            }
            Event::OpenSoundFontPicker => {
                data.is_loading = true;

                let cmd = Task::future(async {
                    Event::SoundFontFileLoaded(open_sound_font_picker().await)
                })
                .map(Message::SettingsPage);
                return PageMessage::Command(cmd);
            }
            Event::SoundFontFileLoaded(font) => {
                if let Some(font) = font {
                    ctx.config.set_soundfont_path(Some(font.clone()));
                }
                data.is_loading = false;
            }
            Event::RangeStart(kind) => match kind {
                RangeUpdateKind::Add => {
                    let v = (ctx.config.piano_range().start() + 1).min(127);
                    if v + 24 < *ctx.config.piano_range().end() {
                        ctx.config.set_piano_range_start(v);
                    }
                }
                RangeUpdateKind::Sub => {
                    ctx.config
                        .set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
                }
            },
            Event::RangeEnd(kind) => match kind {
                RangeUpdateKind::Add => {
                    ctx.config
                        .set_piano_range_end(ctx.config.piano_range().end() + 1);
                }
                RangeUpdateKind::Sub => {
                    let v = ctx.config.piano_range().end().saturating_sub(1);
                    if *ctx.config.piano_range().start() + 24 < v {
                        ctx.config.set_piano_range_end(v);
                    }
                }
            },
            Event::AudioGain(kind) => {
                match kind {
                    RangeUpdateKind::Add => {
                        ctx.config.set_audio_gain(ctx.config.audio_gain() + 0.1);
                    }
                    RangeUpdateKind::Sub => {
                        ctx.config.set_audio_gain(ctx.config.audio_gain() - 0.1);
                    }
                }

                ctx.config
                    .set_audio_gain((ctx.config.audio_gain() * 10.0).round() / 10.0);
            }
        }

        PageMessage::none()
    }

    fn view<'a>(_data: &'a Data, _ctx: &Context) -> Element<'a, Event> {
        row![].into()
    }

    fn keyboard_input(event: &iced_core::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_core::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Tab => Some(Message::SettingsPage(SettingsPage::open_sound_font_picker())),
                Named::Escape => Some(Message::GoBack),
                _ => None,
            },
            _ => None,
        }
    }
}

async fn open_sound_font_picker() -> Option<PathBuf> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("SoundFont2", &["sf2"])
        .pick_file()
        .await;

    if let Some(file) = file.as_ref() {
        log::info!("Font path = {:?}", file.path());
    } else {
        log::info!("User canceled dialog");
    }

    file.map(|f| f.path().to_owned())
}
