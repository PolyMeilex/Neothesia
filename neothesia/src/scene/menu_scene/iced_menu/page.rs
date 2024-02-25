use iced_runtime::Command;
use neothesia_iced_widgets::Element;

use crate::context::Context;

use super::{Data, Message};

pub trait Page {
    type Event;
    fn update(data: &mut Data, msg: Self::Event, ctx: &mut Context) -> Command<Message>;
    fn view<'a>(data: &'a Data, ctx: &Context) -> Element<'a, Self::Event>;
}
