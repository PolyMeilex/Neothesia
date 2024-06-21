use std::path::PathBuf;

use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_runtime::Command;
use iced_widget::{column, container, image, row, text};
use neothesia_iced_widgets::{BarLayout, Layout, NeoBtn};

use crate::{context::Context, scene::menu_scene::icons, song::Song};

use super::{
    page::{Page, PageMessage},
    top_padded, Data, Message, Step,
};

#[derive(Debug, Clone)]
pub enum Event {
    Play,
    GoToPage(Step),
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
            Event::GoToPage(step) => {
                return PageMessage::go_to_page(step);
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

    fn view<'a>(data: &'a Data, ctx: &Context) -> neothesia_iced_widgets::Element<'a, Self::Event> {
        let buttons = column![
            NeoBtn::new_with_label("Select File")
                .on_press(Event::MidiFilePicker(MidiFilePickerMessage::open()))
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
            NeoBtn::new_with_label("Settings")
                .on_press(Event::GoToPage(Step::Settings))
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
            NeoBtn::new_with_label("Exit")
                .on_press(Event::GoToPage(Step::Exit))
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
        ]
        .width(Length::Fixed(450.0))
        .spacing(10);

        let column = column![image(data.logo_handle.clone()), buttons]
            .spacing(40)
            .align_items(Alignment::Center);

        let mut layout = Layout::new().body(top_padded(column));

        if let Some(song) = ctx.song.as_ref() {
            let tracks = NeoBtn::new(
                icons::note_list_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::GoToPage(Step::TrackSelection));

            let play = NeoBtn::new(
                icons::play_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::Play);

            let row = row![tracks, play]
                .spacing(10)
                .align_items(Alignment::Center);

            let container = container(row)
                .width(Length::Fill)
                .align_x(Horizontal::Right)
                .padding(Padding {
                    top: 0.0,
                    right: 10.0,
                    bottom: 10.0,
                    left: 0.0,
                });

            layout = layout.bottom(
                BarLayout::new()
                    .center(
                        text(song.file.name.to_string())
                            .width(Length::Fill)
                            .vertical_alignment(Vertical::Center)
                            .horizontal_alignment(Horizontal::Center),
                    )
                    .right(container),
            );
        }

        layout.into()
    }

    fn keyboard_input(event: &iced_runtime::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

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
    pub(super) fn open() -> Self {
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
) -> Command<MidiFilePickerMessage> {
    match msg {
        MidiFilePickerMessage::OpenMidiFilePicker => {
            data.is_loading = true;

            return Command::perform(
                open_midi_file_picker(),
                MidiFilePickerMessage::MidiFileLoaded,
            );
        }
        MidiFilePickerMessage::MidiFileLoaded(midi) => {
            if let Some((midi, path)) = midi {
                ctx.config.last_opened_song = Some(path);
                ctx.song = Some(Song::new(midi));
            }
            data.is_loading = false;
        }
    }

    Command::none()
}

async fn open_midi_file_picker() -> Option<(midi_file::MidiFile, PathBuf)> {
    let file = rfd::AsyncFileDialog::new()
        .add_filter("midi", &["mid", "midi"])
        .pick_file()
        .await;

    if let Some(file) = file {
        log::info!("File path = {:?}", file.path());

        let thread = async_thread::Builder::new()
            .name("midi-loader".into())
            .spawn(move || {
                let midi = midi_file::MidiFile::new(file.path());

                if let Err(e) = &midi {
                    log::error!("{}", e);
                }

                midi.map(|midi| (midi, file.path().to_path_buf())).ok()
            });

        if let Ok(thread) = thread {
            thread.join().await.ok().flatten()
        } else {
            None
        }
    } else {
        log::info!("User canceled dialog");
        None
    }
}
