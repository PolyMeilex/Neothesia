use super::Renderer;
use iced_core::{
    alignment::{Horizontal, Vertical},
    image::Handle as ImageHandle,
    Alignment, Length,
};
use iced_runtime::Command;
use iced_style::Theme;
use iced_widget::{column as col, container, image, text, vertical_space};

use crate::{
    context::Context,
    iced_utils::iced_state::{Element, Program},
    output_manager::OutputDescriptor,
    scene::menu_scene::iced_menu::main::MainPage,
    NeothesiaEvent,
};

mod exit;
mod main;
mod page;
mod settings;
mod theme;
mod tracks;

use exit::ExitPage;
use page::Page;
use settings::SettingsPage;
use tracks::TracksPage;

type InputDescriptor = midi_io::MidiInputPort;

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    Play,
    GoToPage(Step),
    GoBack,
    ExitApp,

    MainPage(<MainPage as Page>::Event),
    ExitPage(<ExitPage as Page>::Event),
    SettingsPage(<SettingsPage as Page>::Event),
    TracksPage(<TracksPage as Page>::Event),
}

pub struct Data {
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
            Message::GoBack => {
                return self.update(ctx, Message::GoToPage(self.current.previous_step()));
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
            Message::MainPage(msg) => {
                return MainPage::update(&mut self.data, msg, ctx);
            }
            Message::SettingsPage(msg) => {
                return SettingsPage::update(&mut self.data, msg, ctx);
            }
            Message::TracksPage(msg) => {
                return TracksPage::update(&mut self.data, msg, ctx);
            }
            Message::ExitPage(event) => {
                return ExitPage::update(&mut self.data, event, ctx);
            }
            Message::ExitApp => {
                ctx.proxy.send_event(NeothesiaEvent::Exit).ok();
            }
        }

        Command::none()
    }

    fn mouse_input(&self, event: &iced_core::mouse::Event, _ctx: &Context) -> Option<Message> {
        if let iced_core::mouse::Event::ButtonPressed(iced_core::mouse::Button::Back) = event {
            Some(Message::GoBack)
        } else {
            None
        }
    }

    fn keyboard_input(
        &self,
        event: &iced_runtime::keyboard::Event,
        ctx: &Context,
    ) -> Option<Message> {
        match self.current {
            Step::Exit => ExitPage::keyboard_input(event, ctx),
            Step::Main => MainPage::keyboard_input(event, ctx),
            Step::Settings => SettingsPage::keyboard_input(event, ctx),
            Step::TrackSelection => TracksPage::keyboard_input(event, ctx),
        }
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        if self.data.is_loading {
            return Step::loading(&self.data);
        }

        match self.current {
            Step::Exit => ExitPage::view(&self.data, ctx).map(Message::ExitPage),
            Step::Main => MainPage::view(&self.data, ctx).map(Message::MainPage),
            Step::Settings => SettingsPage::view(&self.data, ctx).map(Message::SettingsPage),
            Step::TrackSelection => TracksPage::view(&self.data, ctx).map(Message::TracksPage),
        }
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

    fn loading(data: &'a Data) -> Element<'a, Message> {
        let column = col![image(data.logo_handle.clone()), text("Loading...").size(30)]
            .spacing(40)
            .align_items(Alignment::Center);

        center_x(top_padded(column)).into()
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
