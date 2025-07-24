use iced_widget::row;
use neothesia_iced_widgets::Element;

use crate::{context::Context, NeothesiaEvent};

use super::{page::PageMessage, Data, Message, Page};

pub struct ExitPage;

#[derive(Debug, Clone)]
pub enum Event {
    Exit,
}

impl Page for ExitPage {
    type Event = Event;

    fn update(_data: &mut Data, event: Event, ctx: &mut Context) -> PageMessage {
        match event {
            Event::Exit => {
                ctx.proxy.send_event(NeothesiaEvent::Exit).ok();
            }
        }

        PageMessage::None
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
                Named::Enter => Some(Message::ExitPage(self::Event::Exit)),
                Named::Escape => Some(Message::GoBack),
                _ => None,
            },
            _ => None,
        }
    }
}
