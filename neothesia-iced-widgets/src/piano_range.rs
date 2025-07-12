use iced_core::{
    border::{Border, Radius},
    renderer::Quad,
    Background, Color, Length, Rectangle, Size, Theme, Vector, Widget,
};

use super::Element;

pub struct PianoRange(pub std::ops::RangeInclusive<u8>);

impl<M, R: iced_core::Renderer> Widget<M, Theme, R> for PianoRange {
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

            let mut neutral = layout
                .keys
                .iter()
                .filter(|key| key.kind().is_neutral())
                .enumerate()
                .peekable();

            while let Some((n, key)) = neutral.next() {
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
                            radius: if n == 0 {
                                Radius::new(0.0)
                                    .top_left(12.0)
                                    .top_right(0.0)
                                    .bottom_right(5.0)
                                    .bottom_left(12.0)
                            } else if neutral.peek().is_none() {
                                Radius::new(0.0)
                                    .top_left(0.0)
                                    .top_right(12.0)
                                    .bottom_right(12.0)
                                    .bottom_left(5.0)
                            } else {
                                Radius::new(0.0)
                                    .top_left(0.0)
                                    .top_right(0.0)
                                    .bottom_right(5.0)
                                    .bottom_left(5.0)
                            },
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(Color::WHITE),
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
                            radius: Radius::default(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(Color::BLACK),
                );
            }
        });
    }
}

impl<'a, M: 'a> From<PianoRange> for Element<'a, M> {
    fn from(value: PianoRange) -> Self {
        Self::new(value)
    }
}
