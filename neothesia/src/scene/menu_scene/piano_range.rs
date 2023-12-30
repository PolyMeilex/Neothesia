use iced_core::{renderer::Quad, Background, BorderRadius, Color, Rectangle, Size, Vector, Widget};

use crate::iced_utils::iced_state::Element;

pub struct PianoRange(pub std::ops::RangeInclusive<u8>);

impl<M, R: iced_core::Renderer> Widget<M, R> for PianoRange {
    fn width(&self) -> iced_core::Length {
        iced_core::Length::Fill
    }

    fn height(&self) -> iced_core::Length {
        iced_core::Length::Fixed(100.0)
    }

    fn layout(
        &self,
        _tree: &mut iced_core::widget::Tree,
        _renderer: &R,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        let width = Widget::<M, R>::width(self);
        let height = Widget::<M, R>::height(self);

        let limits = limits.width(width).height(height);
        let size = limits.resolve(Size::ZERO);
        iced_core::layout::Node::new(size)
    }

    fn draw(
        &self,
        _tree: &iced_core::widget::Tree,
        renderer: &mut R,
        _theme: &<R as iced_core::Renderer>::Theme,
        _style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        _cursor: iced_core::mouse::Cursor,
        _viewport: &iced_core::Rectangle,
    ) {
        let bounds = layout.bounds();
        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            let range = piano_math::KeyboardRange::new(self.0.clone());

            let white_count = range.white_count();
            let neutral_width = bounds.width / white_count as f32;
            let neutral_height = bounds.height;

            let layout =
                piano_math::KeyboardLayout::from_range(neutral_width, neutral_height, range);

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
                        border_radius: if n == 0 {
                            BorderRadius::from([12.0, 0.0, 5.0, 12.0])
                        } else if neutral.peek().is_none() {
                            BorderRadius::from([0.0, 12.0, 12.0, 5.0])
                        } else {
                            BorderRadius::from([0.0, 0.0, 5.0, 5.0])
                        },
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
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
                        border_radius: BorderRadius::default(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
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
