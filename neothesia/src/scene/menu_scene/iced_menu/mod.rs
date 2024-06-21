use std::collections::VecDeque;

use self::page::PageMessage;

use super::Renderer;
use iced_core::{
    alignment::{Horizontal, Vertical},
    image::Handle as ImageHandle,
    Alignment, Length, Theme,
};
use iced_runtime::Command;
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
    GoToPage(Step),
    GoBack,

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
    page_stack: VecDeque<Step>,
}

impl AppUi {
    pub fn new(_ctx: &Context) -> Self {
        let mut page_stack = VecDeque::new();
        page_stack.push_front(Step::Main);

        Self {
            page_stack,
            data: Data {
                outputs: Vec::new(),
                selected_output: None,

                inputs: Vec::new(),
                selected_input: None,

                is_loading: false,

                logo_handle: ImageHandle::from_bytes(include_bytes!("../img/banner.png").to_vec()),
            },
        }
    }

    pub fn current(&self) -> &Step {
        self.page_stack.front().unwrap()
    }

    pub fn go_to(&mut self, page: Step) {
        self.page_stack.push_front(page);
    }

    pub fn go_back(&mut self) {
        match self.page_stack.len() {
            1 => {
                // Last page in the stack, let's go to exit page
                self.page_stack.push_front(Step::Exit);
            }
            _ => {
                self.page_stack.pop_front();
            }
        }
    }

    fn handle_page_msg(&mut self, ctx: &mut Context, msg: PageMessage) -> Command<Message> {
        match msg {
            PageMessage::Message(msg) => self.update(ctx, msg),
            PageMessage::Command(cmd) => cmd,
            PageMessage::None => Command::none(),
        }
    }
}

impl Program for AppUi {
    type Message = Message;

    fn update(&mut self, ctx: &mut Context, message: Message) -> Command<Self::Message> {
        match message {
            Message::GoToPage(page) => {
                self.go_to(page);
            }
            Message::GoBack => {
                self.go_back();
            }
            Message::MainPage(msg) => {
                let msg = MainPage::update(&mut self.data, msg, ctx);
                return self.handle_page_msg(ctx, msg);
            }
            Message::SettingsPage(msg) => {
                let msg = SettingsPage::update(&mut self.data, msg, ctx);
                return self.handle_page_msg(ctx, msg);
            }
            Message::TracksPage(msg) => {
                let msg = TracksPage::update(&mut self.data, msg, ctx);
                return self.handle_page_msg(ctx, msg);
            }
            Message::ExitPage(msg) => {
                let msg = ExitPage::update(&mut self.data, msg, ctx);
                return self.handle_page_msg(ctx, msg);
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
        match self.current() {
            Step::Exit => ExitPage::keyboard_input(event, ctx),
            Step::Main => MainPage::keyboard_input(event, ctx),
            Step::Settings => SettingsPage::keyboard_input(event, ctx),
            Step::TrackSelection => TracksPage::keyboard_input(event, ctx),
        }
    }

    fn view(&self, ctx: &Context) -> Element<Message> {
        if self.data.is_loading {
            return loading(&self.data);
        }

        match self.current() {
            Step::Exit => ExitPage::view(&self.data, ctx).map(Message::ExitPage),
            Step::Main => MainPage::view(&self.data, ctx).map(Message::MainPage),
            Step::Settings => SettingsPage::view(&self.data, ctx).map(Message::SettingsPage),
            Step::TrackSelection => TracksPage::view(&self.data, ctx).map(Message::TracksPage),
        }
    }

    fn tick(&mut self, ctx: &mut Context) {
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
}

#[derive(Debug, Clone)]
pub enum Step {
    Exit,
    Main,
    Settings,
    TrackSelection,
}

fn play(data: &Data, ctx: &mut Context) {
    let Some(song) = ctx.song.as_ref() else {
        return;
    };

    if let Some(out) = data.selected_output.clone() {
        let out = match out {
            #[cfg(feature = "synth")]
            OutputDescriptor::Synth(_) => {
                OutputDescriptor::Synth(ctx.config.soundfont_path.clone())
            }
            o => o,
        };

        ctx.output_manager.connect(out);
        ctx.output_manager
            .connection()
            .set_gain(ctx.config.audio_gain);
    }

    if let Some(port) = data.selected_input.clone() {
        ctx.input_manager.connect_input(port);
    }

    ctx.proxy
        .send_event(NeothesiaEvent::Play(song.clone()))
        .ok();
}

fn loading(data: &Data) -> Element<'_, Message> {
    let column = col![image(data.logo_handle.clone()), text("Loading...").size(30)]
        .spacing(40)
        .align_items(Alignment::Center);

    center_x(top_padded(column)).into()
}

fn centered_text<'a>(label: impl ToString) -> iced_widget::Text<'a, Theme, Renderer> {
    text(label.to_string())
        .horizontal_alignment(Horizontal::Center)
        .vertical_alignment(Vertical::Center)
}

fn top_padded<'a, MSG: 'a>(
    content: impl Into<Element<'a, MSG>>,
) -> iced_widget::Column<'a, MSG, Theme, Renderer> {
    let spacer = vertical_space().height(Length::FillPortion(1));
    let content = container(content)
        .height(Length::FillPortion(4))
        .center_x(Length::Fill)
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
        .center_x(Length::Fill)
}
