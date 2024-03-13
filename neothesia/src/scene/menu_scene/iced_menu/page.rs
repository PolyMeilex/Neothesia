use iced_runtime::Command;
use neothesia_iced_widgets::Element;

use crate::context::Context;

use super::{Data, Message, Step};

pub enum PageMessage {
    Message(Message),
    Command(Command<Message>),
    None,
}

impl PageMessage {
    pub fn go_back() -> Self {
        Self::message(Message::GoBack)
    }

    pub fn go_to_page(step: Step) -> Self {
        Self::message(Message::GoToPage(step))
    }

    pub fn none() -> Self {
        Self::None
    }

    fn message(msg: Message) -> Self {
        Self::Message(msg)
    }
}

pub trait Page {
    type Event;
    fn update(data: &mut Data, msg: Self::Event, ctx: &mut Context) -> PageMessage;
    fn view<'a>(data: &'a Data, ctx: &Context) -> Element<'a, Self::Event>;
    fn keyboard_input(event: &iced_runtime::keyboard::Event, ctx: &Context) -> Option<Message>;
}
