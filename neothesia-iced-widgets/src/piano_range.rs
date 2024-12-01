use iced_core::{
    border::{Border, Radius},
    renderer::Quad,
    Background, Color, Length, Rectangle, Size, Theme, Vector, Widget,
    text::Renderer as TextRenderer
};

pub struct PianoRange(pub std::ops::RangeInclusive<u8>);

impl<M, R: iced_core::Renderer + TextRenderer> Widget<M, Theme, R> for PianoRange
where
    <R as iced_core::text::Renderer>::Font: std::default::Default,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: iced_core::Length::Fill,
            height: iced_core::Length::Fixed(100.0),
        }
    }

    fn layout(
        &self,
        _tree: &mut iced_core::widget::Tree,
        _renderer: &R,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let size = Widget::<M, Theme, R>::size(self);
        iced_core::layout::atomic(limits, size.width, size.height)
    }

    fn draw(
        &self,
        _tree: &iced_core::widget::Tree,
        renderer: &mut R,
        _theme: &Theme,
        _style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced_core::Rectangle,
    ) {
        let bounds = layout.bounds();
        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let range = piano_layout::KeyboardRange::new(self.0.clone());

            let white_count = range.white_count();
            let neutral_width = bounds.width / white_count as f32;
            let neutral_height = bounds.height;

            let layout = piano_layout::KeyboardLayout::from_range(
                piano_layout::Sizing::new(neutral_width, neutral_height),
                range,
            );

            for key in layout.keys.iter().filter(|key| key.kind().is_neutral()) {
                let bounds = Rectangle {
                    x: key.x(),
                    y: 0.0,
                    width: key.width(),
                    height: key.height(),
                };

                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: Border {
                            radius: Radius::new(0.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Default::default(),
                    },
                    Background::Color(Color::WHITE),
                );

                let note_id = key.note_id().to_string();
                renderer.fill_text(
                    iced_core::text::Text {
                        content: note_id,
                        bounds: iced_core::Size::new(bounds.width, bounds.height),
                        size: iced_core::Pixels(16.0),
                        font: Default::default(),
                        horizontal_alignment: iced_core::alignment::Horizontal::Center,
                        vertical_alignment: iced_core::alignment::Vertical::Center,
                        line_height: iced_core::text::LineHeight::Absolute(iced_core::Pixels(20.0)),
                        shaping: iced_core::text::Shaping::Basic,
                        wrapping: iced_core::text::Wrapping::default(),
                    },
                    iced_core::Point::new(bounds.x + (bounds.width / 2.0), bounds.y + (bounds.height / 2.0)),
                    iced_core::Color::BLACK,
                    bounds
                );
            }

            for key in layout.keys.iter().filter(|key| key.kind().is_sharp()) {
                let bounds = Rectangle {
                    x: key.x(),
                    y: 0.0,
                    width: key.width(),
                    height: key.height(),
                };

                renderer.fill_quad(
                    Quad {
                        bounds,
                        border: Border {
                            radius: Radius::new(0.0),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Default::default(),
                    },
                    Background::Color(Color::BLACK),
                );

                let note_id = key.note_id().to_string();
                renderer.fill_text(
                    iced_core::text::Text {
                        content: note_id,
                        bounds: iced_core::Size::new(bounds.width, bounds.height),
                        size: iced_core::Pixels(16.0),
                        font: Default::default(),
                        horizontal_alignment: iced_core::alignment::Horizontal::Center,
                        vertical_alignment: iced_core::alignment::Vertical::Center,
                        line_height: iced_core::text::LineHeight::Absolute(iced_core::Pixels(20.0)),
                        shaping: iced_core::text::Shaping::Basic,
                        wrapping: iced_core::text::Wrapping::default(),
                    },
                    iced_core::Point::new(bounds.x + (bounds.width / 2.0), bounds.y + (bounds.height / 2.0)),
                    iced_core::Color::WHITE,
                    bounds
                );
            }
        });
    }
}
