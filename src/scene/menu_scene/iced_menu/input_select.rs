use iced_native::{
    alignment::Horizontal, alignment::Vertical, Alignment, Color, Column, Container, Element,
    Length, Row, Text,
};
use iced_wgpu::Renderer;
use midir::{MidiInput, MidiInputPort};

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
        out_carousel: &mut Carousel<MidiInputPort>,
        midi_file: bool,
        play_along: bool,
    ) -> (Element<Message, Renderer>, Element<Message, Renderer>) {
        let item = out_carousel.get_item();

        let midi_in = MidiInput::new("midi_in").unwrap();

        let label = item
            .map(|o| {
                midi_in
                    .port_name(o)
                    .unwrap_or("Error".to_string())
                    .to_string()
            })
            .unwrap_or_else(|| "Disconnected".to_string());

        let output = Text::new(label)
            .color(Color::WHITE)
            .size(30)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let mut select_row = Row::new().height(Length::Units(50)).push(
            NeoBtn::new(
                &mut self.prev_button,
                Text::new("<")
                    .size(40)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .disabled(!out_carousel.check_prev())
            .on_press(Message::PrevPressed),
        );

        select_row = select_row.push(
            NeoBtn::new(
                &mut self.next_button,
                Text::new(">")
                    .size(40)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .disabled(!out_carousel.check_next())
            .on_press(Message::NextPressed),
        );

        {
            let controls = Column::new()
                .align_items(Alignment::Center)
                .width(Length::Units(500))
                .spacing(30)
                .push(output)
                .push(select_row);

            (
                Container::new(controls)
                    .width(Length::Fill)
                    .height(Length::Units(250))
                    .center_x()
                    .center_y()
                    .into(),
                Self::footer(&mut self.play_button, &out_carousel, midi_file, play_along),
            )
        }
    }

    #[allow(unused_variables)]
    fn footer<'a>(
        play_button: &'a mut neo_btn::State,
        in_carousel: &Carousel<MidiInputPort>,
        midi_file: bool,
        play_along: bool,
    ) -> Element<'a, Message, Renderer> {
        let content: Element<Message, Renderer> = if midi_file && in_carousel.get_item().is_some() {
            let btn = NeoBtn::new(
                play_button,
                Text::new("Play")
                    .size(30)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center)
                    .color(Color::WHITE),
            )
            .min_height(50)
            .height(Length::Fill)
            .width(Length::Units(150))
            .on_press(Message::EnterPressed);

            #[allow(unused_mut)]
            let mut coll = Column::new().spacing(10);

            coll.push(btn).into()
        } else {
            Row::new().into()
        };

        Container::new(content)
            .width(Length::Fill)
            .height(Length::Units(100))
            .padding(10)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Bottom)
            .into()
    }
}
