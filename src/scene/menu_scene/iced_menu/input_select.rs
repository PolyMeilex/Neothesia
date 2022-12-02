use iced_native::{
    alignment::{Horizontal, Vertical},
    widget::{
        helpers::{column, container, row, text},
        Row,
    },
    Alignment, Color, Element, Length,
};
use iced_wgpu::Renderer;

use crate::scene::menu_scene::neo_btn::{self, NeoBtn};

use super::{carousel::Carousel, Message};

#[derive(Default)]
pub struct InputSelectControls {
    prev_button: neo_btn::State,
    next_button: neo_btn::State,
    play_button: neo_btn::State,
}

impl InputSelectControls {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn view(
        &mut self,
        in_carousel: &mut Carousel<midi_io::MidiInputPort>,
        midi_file: bool,
        play_along: bool,
    ) -> (Element<Message, Renderer>, Element<Message, Renderer>) {
        let item = in_carousel.get_item();

        let label = item
            .map(|o| o.to_string())
            .unwrap_or_else(|| "Disconnected".to_string());

        let title = text("Select Input:")
            .style(Color::WHITE)
            .size(30)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let output = text(label)
            .style(Color::WHITE)
            .size(30)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let mut select_row = Row::new().height(Length::Units(50)).push(
            NeoBtn::new(
                &mut self.prev_button,
                text("<")
                    .size(40)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .disabled(!in_carousel.check_prev())
            .on_press(Message::PrevPressed),
        );

        select_row = select_row.push(
            NeoBtn::new(
                &mut self.next_button,
                text(">")
                    .size(40)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .disabled(!in_carousel.check_next())
            .on_press(Message::NextPressed),
        );

        let controls = column(vec![title.into(), output.into(), select_row.into()])
            .align_items(Alignment::Center)
            .width(Length::Units(500))
            .spacing(30);

        (
            container(controls)
                .width(Length::Fill)
                .height(Length::Units(250))
                .center_x()
                .center_y()
                .into(),
            Self::footer(&mut self.play_button, in_carousel, midi_file, play_along),
        )
    }

    #[allow(unused_variables)]
    fn footer<'a>(
        play_button: &'a mut neo_btn::State,
        in_carousel: &Carousel<midi_io::MidiInputPort>,
        midi_file: bool,
        play_along: bool,
    ) -> Element<'a, Message, Renderer> {
        let content: Element<Message, Renderer> = if midi_file && in_carousel.get_item().is_some() {
            let btn = NeoBtn::new(
                play_button,
                text("Play")
                    .size(30)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .style(Color::WHITE),
            )
            .min_height(50)
            .height(Length::Fill)
            .width(Length::Units(150))
            .on_press(Message::ContinuePressed);

            column(vec![btn.into()]).spacing(10).into()
        } else {
            row(vec![]).into()
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Units(100))
            .padding(10)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Bottom)
            .into()
    }
}
