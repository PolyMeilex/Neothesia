use iced_runtime::Task;
use neothesia_iced_widgets::Element;

use crate::context::Context;

use super::{Data, Message};

pub enum PageMessage {
    Command(Task<Message>),
    None,
}

impl PageMessage {
    pub fn none() -> Self {
        Self::None
    }
}

pub trait Page {
    type Event;
    fn update(data: &mut Data, msg: Self::Event, ctx: &mut Context) -> PageMessage;
    fn view<'a>(data: &'a Data, ctx: &Context) -> Element<'a, Self::Event>;
    fn keyboard_input(event: &iced_core::keyboard::Event, ctx: &Context) -> Option<Message>;
}
