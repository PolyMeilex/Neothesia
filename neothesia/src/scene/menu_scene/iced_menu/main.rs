use std::path::PathBuf;

use iced_runtime::Task;
use neothesia_iced_widgets::Layout;

use crate::{context::Context, song::Song};

use super::{
    page::{Page, PageMessage},
    Data, Message, Step,
};

#[derive(Debug, Clone)]
pub enum Event {
    Play,
    MidiFilePicker(MidiFilePickerMessage),
}

pub struct MainPage;

impl Page for MainPage {
    type Event = Event;

    fn update(data: &mut Data, msg: Self::Event, ctx: &mut Context) -> PageMessage {
        match msg {
            Event::Play => {
                super::play(data, ctx);
            }
            Event::MidiFilePicker(msg) => {
                return PageMessage::Command(
                    midi_file_picker_update(data, msg, ctx)
                        .map(Event::MidiFilePicker)
                        .map(Message::MainPage),
                );
            }
        };

        PageMessage::None
    }

    fn view<'a>(
        _data: &'a Data,
        _ctx: &Context,
    ) -> neothesia_iced_widgets::Element<'a, Self::Event> {
        Layout::new().into()
    }

    fn keyboard_input(event: &iced_core::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_core::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Tab => Some(MidiFilePickerMessage::open().into()),
                Named::Enter => Some(Message::MainPage(self::Event::Play)),
                Named::Escape => Some(Message::GoBack),
                _ => None,
            },
            Event::KeyPressed {
                key: Key::Character(ch),
                ..
            } => match ch.as_ref() {
                "s" => Some(Message::GoToPage(Step::Settings)),
                "t" => Some(Message::GoToPage(Step::TrackSelection)),
                _ => None,
            },
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MidiFilePickerMessage {
    OpenMidiFilePicker,
    MidiFileLoaded(Option<(midi_file::MidiFile, PathBuf)>),
}

impl MidiFilePickerMessage {
    pub fn open() -> Self {
        Self::OpenMidiFilePicker
    }
}

impl From<MidiFilePickerMessage> for Message {
    fn from(msg: MidiFilePickerMessage) -> Self {
        Message::MainPage(Event::MidiFilePicker(msg))
    }
}

fn midi_file_picker_update(
    data: &mut Data,
    msg: MidiFilePickerMessage,
    ctx: &mut Context,
) -> Task<MidiFilePickerMessage> {
    match msg {
        MidiFilePickerMessage::OpenMidiFilePicker => {
            data.is_loading = true;

            return Task::future(async {
                MidiFilePickerMessage::MidiFileLoaded(open_midi_file_picker().await)
            });
        }
        MidiFilePickerMessage::MidiFileLoaded(midi) => {
            if let Some((midi, path)) = midi {
                ctx.config.set_last_opened_song(Some(path));
                data.song = Some(Song::new(midi));
            }
            data.is_loading = false;
        }
    }

    Task::none()
}

async fn open_midi_file_picker() -> Option<(midi_file::MidiFile, PathBuf)> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("midi", &["mid", "midi"])
        .pick_file()
        .await;

    if let Some(file) = file {
        log::info!("File path = {:?}", file.path());

        let thread = iced_runtime::task::thread::spawn("midi-loader".into(), move || {
            let midi = midi_file::MidiFile::new(file.path());

            if let Err(e) = &midi {
                log::error!("{e}");
            }

            midi.map(|midi| (midi, file.path().to_path_buf())).ok()
        });

        thread.join().await.ok().flatten()
    } else {
        log::info!("User canceled dialog");
        None
    }
}
