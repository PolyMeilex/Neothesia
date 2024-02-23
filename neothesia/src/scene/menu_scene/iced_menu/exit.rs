use iced_core::{Alignment, Length};
use iced_runtime::Command;
use iced_widget::{column as col, row};
use neothesia_iced_widgets::NeoBtn;

use crate::{context::Context, iced_utils::iced_state::Element, NeothesiaEvent};

use super::{center_x, centered_text, Data, Message, Step};

pub(super) fn update(_data: &mut Data, _msg: (), ctx: &mut Context) -> Command<Message> {
    ctx.proxy.send_event(NeothesiaEvent::Exit).ok();
    Command::none()
}

pub(super) fn view<'a>(_data: &'a Data, _ctx: &Context) -> Element<'a, Message> {
    let output = centered_text("Do you want to exit?").size(30);

    let select_row = row![
        NeoBtn::new_with_label("No")
            .width(Length::Fill)
            .on_press(Message::GoToPage(Step::Main)),
        NeoBtn::new_with_label("Yes")
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
