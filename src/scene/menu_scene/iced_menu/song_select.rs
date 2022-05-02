use iced_graphics::{
    alignment::{Horizontal, Vertical},
    Alignment, Color,
};
use iced_native::widget::{Column, Container, Row, Text};
use iced_native::{Element, Length};
use iced_wgpu::Renderer;

use crate::{
    output_manager::OutputDescriptor,
    scene::menu_scene::neo_btn::{self, NeoBtn},
};

use super::{carousel::Carousel, Message};

#[derive(Default)]
pub struct SongSelectControls {
    file_select_button: neo_btn::State,
    synth_button: neo_btn::State,
    prev_button: neo_btn::State,
    next_button: neo_btn::State,
    play_button: neo_btn::State,
}

impl SongSelectControls {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn view(
        &mut self,
        out_carousel: &mut Carousel<OutputDescriptor>,
        midi_file: bool,
        play_along: bool,
    ) -> (Element<Message, Renderer>, Element<Message, Renderer>) {
        let file_select_button = Row::new().height(Length::Units(100)).push(
            NeoBtn::new(
                &mut self.file_select_button,
                Text::new("Select File")
                    .color(Color::WHITE)
                    .size(40)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .on_press(Message::FileSelectPressed),
        );

        let item = out_carousel.get_item();

        let label = item
            .map(|o| o.to_string())
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

        #[cfg(feature = "synth")]
        if let Some(OutputDescriptor::Synth(_)) = item {
            select_row = select_row.push(
                NeoBtn::new(
                    &mut self.synth_button,
                    Text::new("Soundfont")
                        .size(20)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .width(Length::Units(100))
                .height(Length::Fill)
                .on_press(Message::FontSelectPressed),
            );
        }

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

        let controls = Column::new()
            .align_items(Alignment::Center)
            .width(Length::Units(500))
            .height(Length::Units(250))
            .spacing(30)
            .push(file_select_button)
            .push(output)
            .push(select_row);

        (
            Container::new(controls)
                .width(Length::Fill)
                .center_x()
                .into(),
            Self::footer(&mut self.play_button, out_carousel, midi_file, play_along),
        )
    }

    #[allow(unused_variables)]
    pub fn footer<'a>(
        play_button: &'a mut neo_btn::State,
        out_carousel: &Carousel<OutputDescriptor>,
        midi_file: bool,
        play_along: bool,
    ) -> Element<'a, Message, Renderer> {
        let content: Element<Message, Renderer> = if midi_file && out_carousel.get_item().is_some()
        {
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

            #[cfg(feature = "play_along")]
            {
                use iced_native::widget::Checkbox;
                coll = coll.push(
                    Row::new()
                        .height(Length::Shrink)
                        .push(
                            Checkbox::new(play_along, "", Message::TogglePlayAlong)
                                .style(CheckboxStyle {}),
                        )
                        .push(Text::new("Play Along").color(Color::WHITE)),
                );
            }

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

pub struct CheckboxStyle;

const SURFACE: Color = Color::from_rgb(
    0x30 as f32 / 255.0,
    0x34 as f32 / 255.0,
    0x3B as f32 / 255.0,
);

impl iced_style::checkbox::StyleSheet for CheckboxStyle {
    fn active(&self, is_checked: bool) -> iced_style::checkbox::Style {
        let active = Color::from_rgba8(160, 81, 255, 1.0);
        iced_style::checkbox::Style {
            background: if is_checked { active } else { SURFACE }.into(),
            text_color: Some(Color::WHITE),
            checkmark_color: Color::WHITE,
            border_radius: 2.0,
            border_width: 1.0,
            border_color: active,
        }
    }

    fn hovered(&self, is_checked: bool) -> iced_style::checkbox::Style {
        let active = Color::from_rgba8(160, 81, 255, 1.0);
        iced_style::checkbox::Style {
            background: Color {
                a: 0.8,
                ..if is_checked { active } else { SURFACE }
            }
            .into(),
            ..self.active(is_checked)
        }
    }
}
