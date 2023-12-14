use std::path::PathBuf;

use super::Renderer;
use iced_core::{
    alignment::{Horizontal, Vertical},
    image::Handle as ImageHandle,
    Alignment, Length, Padding,
};
use iced_runtime::Command;
use iced_widget::{button, column as col, container, image, row, text, vertical_space};

use crate::{
    iced_utils::iced_state::{Element, Program},
    output_manager::OutputDescriptor,
    scene::menu_scene::neo_btn::neo_button,
    song::Song,
    target::Target,
    NeothesiaEvent,
};

mod settings;
mod theme;
mod tracks;

use settings::SettingsMessage;
use tracks::TracksMessage;

type InputDescriptor = midi_io::MidiInputPort;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,

    Settings(SettingsMessage),
    Tracks(TracksMessage),

    OpenMidiFilePicker,
    MidiFileLoaded(Option<(midi_file::MidiFile, PathBuf)>),

    Play,

    GoToPage(Step),
    ExitApp,
}

struct Data {
    outputs: Vec<OutputDescriptor>,
    selected_output: Option<OutputDescriptor>,

    inputs: Vec<InputDescriptor>,
    selected_input: Option<InputDescriptor>,

    is_loading: bool,

    logo_handle: ImageHandle,
}

pub struct AppUi {
    data: Data,
    current: Step,
}

impl AppUi {
    pub fn new(_target: &Target) -> Self {
        Self {
            current: Step::Main,
            data: Data {
                outputs: Vec::new(),
                selected_output: None,

                inputs: Vec::new(),
                selected_input: None,

                is_loading: false,

                logo_handle: ImageHandle::from_memory(include_bytes!("../img/banner.png").to_vec()),
            },
        }
    }
}

impl Program for AppUi {
    type Message = Message;

    fn update(&mut self, target: &mut Target, message: Message) -> Command<Self::Message> {
        match message {
            Message::GoToPage(page) => {
                self.current = page;
            }
            Message::Play => {
                if let Some(song) = target.song.as_ref() {
                    if let Some(out) = self.data.selected_output.clone() {
                        let out = match out {
                            #[cfg(feature = "synth")]
                            OutputDescriptor::Synth(_) => {
                                OutputDescriptor::Synth(target.config.soundfont_path.clone())
                            }
                            o => o,
                        };

                        target.output_manager.borrow_mut().connect(out)
                    }

                    if let Some(port) = self.data.selected_input.clone() {
                        target.input_manager.connect_input(port);
                    }

                    target
                        .proxy
                        .send_event(NeothesiaEvent::Play(song.clone()))
                        .ok();
                }
            }
            Message::OpenMidiFilePicker => {
                self.data.is_loading = true;
                return open_midi_file_picker(Message::MidiFileLoaded);
            }
            Message::MidiFileLoaded(midi) => {
                if let Some((midi, path)) = midi {
                    target.config.last_opened_song = Some(path);
                    target.song = Some(Song::new(midi));
                }
                self.data.is_loading = false;
            }

            Message::Settings(msg) => {
                return settings::update(&mut self.data, msg, target);
            }
            Message::Tracks(msg) => {
                return tracks::update(&mut self.data, msg, target);
            }

            Message::Tick => {
                self.data.outputs = target.output_manager.borrow().outputs();
                self.data.inputs = target.input_manager.inputs();

                if self.data.selected_output.is_none() {
                    if let Some(out) = self
                        .data
                        .outputs
                        .iter()
                        .find(|output| Some(output.to_string()) == target.config.output)
                    {
                        self.data.selected_output = Some(out.clone());
                    } else {
                        self.data.selected_output = self.data.outputs.first().cloned();
                    }
                }

                if self.data.selected_input.is_none() {
                    if let Some(input) = self
                        .data
                        .inputs
                        .iter()
                        .find(|input| Some(input.to_string()) == target.config.input)
                    {
                        self.data.selected_input = Some(input.clone());
                    } else {
                        self.data.selected_input = self.data.inputs.first().cloned();
                    }
                }
            }
            Message::ExitApp => {
                target.proxy.send_event(NeothesiaEvent::Exit).ok();
            }
        }

        Command::none()
    }

    fn keyboard_input(
        &self,
        event: &iced_runtime::keyboard::Event,
        _target: &Target,
    ) -> Option<Message> {
        use iced_runtime::keyboard::{Event, KeyCode};

        if let Event::KeyPressed { key_code, .. } = event {
            match key_code {
                KeyCode::Tab => match self.current {
                    Step::Main => Some(Message::OpenMidiFilePicker),
                    Step::Settings => Some(Message::Settings(SettingsMessage::OpenSoundFontPicker)),
                    _ => None,
                },
                KeyCode::S => match self.current {
                    Step::Main => Some(Message::GoToPage(Step::Settings)),
                    _ => None,
                },
                KeyCode::Enter => match self.current {
                    Step::Exit => Some(Message::ExitApp),
                    Step::Main => Some(Message::Play),
                    _ => None,
                },
                // Let's hide track screen behind a magic key, as it's not ready for prime time
                KeyCode::T => match self.current {
                    Step::Main => Some(Message::GoToPage(Step::TrackSelection)),
                    _ => None,
                },
                KeyCode::Escape => Some(match self.current {
                    Step::Exit => Message::GoToPage(Step::Main),
                    Step::Main => Message::GoToPage(Step::Exit),
                    Step::Settings => Message::GoToPage(Step::Main),
                    Step::TrackSelection => Message::GoToPage(Step::Main),
                }),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(&self, target: &Target) -> Element<Message> {
        self.current.view(&self.data, target)
    }
}

#[derive(Debug, Clone)]
pub enum Step {
    Exit,
    Main,
    Settings,
    TrackSelection,
}

impl<'a> Step {
    fn view(&'a self, data: &'a Data, target: &Target) -> Element<Message> {
        if data.is_loading {
            return Self::loading(data);
        }

        match self {
            Self::Exit => Self::exit(),
            Self::Main => Self::main(data, target),
            Self::Settings => settings::view(data, target),
            Self::TrackSelection => tracks::view(data, target),
        }
    }

    fn loading(data: &'a Data) -> Element<'a, Message> {
        let column = col![image(data.logo_handle.clone()), text("Loading...").size(30)]
            .spacing(40)
            .align_items(Alignment::Center);

        center_x(top_padded(column)).into()
    }

    fn exit() -> Element<'a, Message> {
        let output = centered_text("Do you want to exit?").size(30);

        let select_row = row![
            neo_button("No")
                .width(Length::Fill)
                .on_press(Message::GoToPage(Step::Main)),
            neo_button("Yes")
                .width(Length::Fill)
                .on_press(Message::ExitApp),
        ]
        .spacing(5)
        .height(Length::Fixed(50.0));

        let controls = col![output, select_row]
            .align_items(Alignment::Center)
            .width(Length::Fixed(650.0))
            .spacing(30);

        center_x(controls).center_y().into()
    }

    fn main(data: &'a Data, target: &Target) -> Element<'a, Message> {
        let buttons = col![
            neo_button("Select File")
                .on_press(Message::OpenMidiFilePicker)
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
            neo_button("Settings")
                .on_press(Message::GoToPage(Step::Settings))
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
            neo_button("Exit")
                .on_press(Message::GoToPage(Step::Exit))
                .width(Length::Fill)
                .height(Length::Fixed(80.0))
        ]
        .width(Length::Fixed(450.0))
        .spacing(10);

        let column = col![image(data.logo_handle.clone()), buttons]
            .spacing(40)
            .align_items(Alignment::Center);

        let mut content = top_padded(column);

        if target.song.is_some() {
            let tracks = button(centered_text("Tracks"))
                .on_press(Message::GoToPage(Step::TrackSelection))
                .style(theme::button());

            let play = neo_button("Play")
                .height(Length::Fixed(60.0))
                .min_width(80.0)
                .on_press(Message::Play);

            let row = row![tracks, play]
                .spacing(20)
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

            content = content.push(container);
        }

        center_x(content).into()
    }
}

fn centered_text<'a>(label: impl ToString) -> iced_widget::Text<'a, Renderer> {
    text(label)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
}

fn top_padded<'a, MSG: 'a>(
    content: impl Into<Element<'a, MSG>>,
) -> iced_widget::Column<'a, MSG, Renderer> {
    let spacer = vertical_space(Length::FillPortion(1));
    let content = container(content)
        .height(Length::FillPortion(4))
        .center_x()
        .max_width(650);

    col![spacer, content]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_items(Alignment::Center)
}

fn center_x<'a, MSG: 'a>(
    content: impl Into<Element<'a, MSG>>,
) -> iced_widget::Container<'a, MSG, Renderer> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
}

fn open_midi_file_picker(
    f: impl FnOnce(Option<(midi_file::MidiFile, PathBuf)>) -> Message + 'static + Send,
) -> Command<Message> {
    Command::perform(
        async {
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
        },
        f,
    )
}

fn open_sound_font_picker(
    f: impl FnOnce(Option<PathBuf>) -> Message + 'static + Send,
) -> Command<Message> {
    Command::perform(
        async {
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
        },
        f,
    )
}
