use super::{icons, Renderer};
use iced_core::{
    alignment::{Horizontal, Vertical},
    image::Handle as ImageHandle,
    Alignment, Length, Padding,
};
use iced_runtime::Command;
use iced_style::Theme;
use iced_widget::{column as col, container, image, row, text, vertical_space};
use neothesia_iced_widgets::{BarLayout, Layout, NeoBtn};

use crate::{
    context::Context,
    iced_utils::iced_state::{Element, Program},
    output_manager::OutputDescriptor,
    NeothesiaEvent,
};

mod exit;
mod midi_file_picker;
mod settings;
mod theme;
mod tracks;

use midi_file_picker::MidiFilePickerMessage;
use settings::SettingsMessage;
use tracks::TracksMessage;

type InputDescriptor = midi_io::MidiInputPort;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Play,
    GoToPage(Step),
    ExitApp,

    Settings(SettingsMessage),
    Tracks(TracksMessage),
    MidiFilePicker(MidiFilePickerMessage),
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
    pub fn new(_ctx: &Context) -> Self {
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

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Self::Message> {
        match message {
            Message::GoToPage(page) => {
                self.current = page;
            }
            Message::Play => {
                if let Some(song) = ctx.song.as_ref() {
                    if let Some(out) = self.data.selected_output.clone() {
                        let out = match out {
                            #[cfg(feature = "synth")]
                            OutputDescriptor::Synth(_) => {
                                OutputDescriptor::Synth(ctx.config.soundfont_path.clone())
                            }
                            o => o,
                        };

                        ctx.output_manager.connect(out)
                    }

                    if let Some(port) = self.data.selected_input.clone() {
                        ctx.input_manager.connect_input(port);
                    }

                    ctx.proxy
                        .send_event(NeothesiaEvent::Play(song.clone()))
                        .ok();
                }
            }
            Message::Tick => {
                self.data.outputs = ctx.output_manager.outputs();
                self.data.inputs = ctx.input_manager.inputs();

                if self.data.selected_output.is_none() {
                    if let Some(out) = self
                        .data
                        .outputs
                        .iter()
                        .find(|output| Some(output.to_string()) == ctx.config.output)
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
                        .find(|input| Some(input.to_string()) == ctx.config.input)
                    {
                        self.data.selected_input = Some(input.clone());
                    } else {
                        self.data.selected_input = self.data.inputs.first().cloned();
                    }
                }
            }
            Message::Settings(msg) => {
                return settings::update(&mut self.data, msg, ctx);
            }
            Message::Tracks(msg) => {
                return tracks::update(&mut self.data, msg, ctx);
            }
            Message::MidiFilePicker(msg) => {
                return midi_file_picker::update(&mut self.data, msg, ctx);
            }
            Message::ExitApp => {
                return exit::update(&mut self.data, (), ctx);
            }
        }

        Command::none()
    }

    fn mouse_input(&self, event: &iced_core::mouse::Event, _ctx: &Context) -> Option<Message> {
        if let iced_core::mouse::Event::ButtonPressed(iced_core::mouse::Button::Back) = event {
            Some(Message::GoToPage(self.current.previous_step()))
        } else {
            None
        }
    }

    fn keyboard_input(
        &self,
        event: &iced_runtime::keyboard::Event,
        _ctx: &Context,
    ) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Tab => match self.current {
                    Step::Main => Some(midi_file_picker::open().into()),
                    Step::Settings => Some(Message::Settings(SettingsMessage::OpenSoundFontPicker)),
                    _ => None,
                },
                Named::Enter => match self.current {
                    Step::Exit => Some(Message::ExitApp),
                    Step::Main => Some(Message::Play),
                    Step::TrackSelection => Some(Message::Play),
                    _ => None,
                },
                Named::Escape => Some(Message::GoToPage(self.current.previous_step())),
                _ => None,
            },
            Event::KeyPressed {
                key: Key::Character(ch),
                ..
            } => match ch.as_ref() {
                "s" => match self.current {
                    Step::Main => Some(Message::GoToPage(Step::Settings)),
                    _ => None,
                },
                "t" => match self.current {
                    Step::Main => Some(Message::GoToPage(Step::TrackSelection)),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        self.current.view(&self.data, ctx)
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
    fn previous_step(&self) -> Self {
        match self {
            Step::Exit => Step::Main,
            Step::Main => Step::Exit,
            Step::Settings => Step::Main,
            Step::TrackSelection => Step::Main,
        }
    }

    fn view(&'a self, data: &'a Data, ctx: &Context) -> Element<Message> {
        if data.is_loading {
            return Self::loading(data);
        }

        match self {
            Self::Exit => exit::view(data, ctx),
            Self::Main => Self::main(data, ctx),
            Self::Settings => settings::view(data, ctx),
            Self::TrackSelection => tracks::view(data, ctx),
        }
    }

    fn loading(data: &'a Data) -> Element<'a, Message> {
        let column = col![image(data.logo_handle.clone()), text("Loading...").size(30)]
            .spacing(40)
            .align_items(Alignment::Center);

        center_x(top_padded(column)).into()
    }

    fn main(data: &'a Data, ctx: &Context) -> Element<'a, Message> {
        let buttons = col![
            NeoBtn::new_with_label("Select File")
                .on_press(midi_file_picker::open().into())
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
            NeoBtn::new_with_label("Settings")
                .on_press(Message::GoToPage(Step::Settings))
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
            NeoBtn::new_with_label("Exit")
                .on_press(Message::GoToPage(Step::Exit))
                .width(Length::Fill)
                .height(Length::Fixed(80.0)),
        ]
        .width(Length::Fixed(450.0))
        .spacing(10);

        let column = col![image(data.logo_handle.clone()), buttons]
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
            .on_press(Message::GoToPage(Step::TrackSelection));

            let play = NeoBtn::new(
                icons::play_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Message::Play);

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
                        text(&song.file.name)
                            .width(Length::Fill)
                            .vertical_alignment(Vertical::Center)
                            .horizontal_alignment(Horizontal::Center),
                    )
                    .right(container),
            );
        }

        layout.into()
    }
}

fn centered_text<'a>(label: impl ToString) -> iced_widget::Text<'a, Theme, Renderer> {
    text(label)
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
}

fn top_padded<'a, MSG: 'a>(
    content: impl Into<Element<'a, MSG>>,
) -> iced_widget::Column<'a, MSG, Theme, Renderer> {
    let spacer = vertical_space().height(Length::FillPortion(1));
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
) -> iced_widget::Container<'a, MSG, Theme, Renderer> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
}
