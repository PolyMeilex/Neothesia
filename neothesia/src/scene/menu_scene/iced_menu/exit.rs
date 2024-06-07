use iced_core::{Alignment, Length};
use iced_widget::{column as col, row};
use neothesia_iced_widgets::{Element, NeoBtn};

use crate::{context::Context, NeothesiaEvent};

use super::{center_x, centered_text, page::PageMessage, Data, Message, Page};

pub struct ExitPage;

#[derive(Debug, Clone)]
pub enum Event {
    GoBack,
    Exit,
}

impl Page for ExitPage {
    type Event = Event;

    fn update(_data: &mut Data, event: Event, ctx: &mut Context) -> PageMessage {
        match event {
            Event::GoBack => {
                return PageMessage::go_back();
            }
            Event::Exit => {
                ctx.proxy.send_event(NeothesiaEvent::Exit).ok();
            }
        }

        PageMessage::None
    }

    fn view<'a>(_data: &'a Data, _ctx: &Context) -> Element<'a, Event> {
        let output = centered_text("Do you want to exit?").size(30);

        let select_row = row![
            NeoBtn::new_with_label("No")
                .width(Length::Fill)
                .on_press(Event::GoBack),
            NeoBtn::new_with_label("Yes")
                .width(Length::Fill)
                .on_press(Event::Exit),
        ]
        .spacing(5)
        .height(Length::Fixed(50.0));

        let controls = col![output, select_row]
            .align_items(Alignment::Center)
            .width(Length::Fixed(650.0))
            .spacing(30);

        center_x(controls).center_y(Length::Fill).into()
    }

    fn keyboard_input(event: &iced_runtime::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

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
