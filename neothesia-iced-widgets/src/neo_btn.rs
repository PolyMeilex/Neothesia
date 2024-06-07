use super::Renderer;
use iced_core::{
    alignment::{Horizontal, Vertical},
    border::Radius,
    layout, mouse,
    renderer::Style,
    widget::{tree, Tree},
    Background, Border, Clipboard, Color, Element, Event, Layout, Length, Padding, Rectangle,
    Shell, Size, Theme, Widget,
};
use iced_widget::text;

pub struct NeoBtn<'a, Message> {
    width: Length,
    height: Length,
    min_width: f32,
    min_height: f32,
    padding: f32,
    border_radius: f32,
    disabled: bool,
    content: Element<'a, Message, Theme, Renderer>,
    on_press: Option<Message>,
}

impl<'a, Message: Clone> NeoBtn<'a, Message> {
    pub fn new_with_label(label: &'a str) -> Self {
        Self::new(
            text(label)
                .size(30)
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center),
        )
    }

    pub fn new<E>(content: E) -> Self
    where
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        Self {
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0.0,
            min_height: 0.0,
            padding: 5.0,
            border_radius: 7.0,
            disabled: false,
            content: content.into(),
        }
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        self.min_height = min_height;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_press(mut self, msg: Message) -> Self {
        self.on_press = Some(msg);
        self
    }
}

impl<'a, Message: Clone> Widget<Message, Theme, Renderer> for NeoBtn<'a, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.min_width(self.min_width).min_height(self.min_height);

        iced_core::layout::padded(
            &limits,
            self.width,
            self.height,
            Padding::new(self.padding),
            |limits| {
                self.content
                    .as_widget()
                    .layout(&mut tree.children[0], renderer, limits)
            },
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor
            .position()
            .map(|point| layout.bounds().contains(point))
            .unwrap_or(false);

        if is_mouse_over && !self.disabled {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> iced_core::event::Status {
        if self.disabled {
            return iced_core::event::Status::Ignored;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.on_press.is_some() {
                    let is_mouse_over = cursor
                        .position()
                        .map(|point| layout.bounds().contains(point))
                        .unwrap_or(false);

                    tree.state.downcast_mut::<State>().is_pressed = is_mouse_over;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(on_press) = self.on_press.clone() {
                    let is_pressed = &mut tree.state.downcast_mut::<State>().is_pressed;

                    let is_mouse_over = cursor
                        .position()
                        .map(|point| layout.bounds().contains(point))
                        .unwrap_or(false);

                    let is_clicked = *is_pressed && is_mouse_over;

                    *is_pressed = false;

                    if is_clicked {
                        shell.publish(on_press);
                    }
                }
            }
            _ => {}
        };

        iced_core::event::Status::Ignored
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let is_mouse_over = cursor
            .position()
            .map(|point| bounds.contains(point))
            .unwrap_or(false);

        let colors = if is_mouse_over {
            (
                Color::from_rgba8(9, 9, 9, 0.6),
                Color::from_rgba8(56, 145, 255, 1.0),
            )
        } else {
            (
                Color::from_rgba8(17, 17, 17, 0.6),
                Color::from_rgba8(160, 81, 255, 1.0),
            )
        };

        use iced_core::renderer::Renderer;
        renderer.fill_quad(
            iced_core::renderer::Quad {
                bounds: Rectangle {
                    y: bounds.y,
                    ..bounds
                },
                border: Border {
                    radius: Radius::from(self.border_radius),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: iced_core::Shadow::default(),
            },
            Background::Color(colors.0),
        );

        renderer.start_layer(Rectangle {
            y: bounds.y + bounds.height - self.border_radius,
            height: self.border_radius,
            ..bounds
        });
        renderer.fill_quad(
            iced_core::renderer::Quad {
                bounds: Rectangle {
                    y: bounds.y,
                    ..bounds
                },
                border: Border {
                    radius: Radius::from(self.border_radius),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                shadow: iced_core::Shadow::default(),
            },
            Background::Color(colors.1),
        );
        renderer.end_layer();

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        };

        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &Style {
                text_color: if self.disabled {
                    Color::new(0.3, 0.3, 0.3, 1.0)
                } else {
                    Color::WHITE
                },
            },
            layout,
            cursor,
            viewport,
        );
    }
}

impl<'a, Message> From<NeoBtn<'a, Message>> for Element<'a, Message, Theme, Renderer>
where
    Message: 'a + Clone,
{
    fn from(from: NeoBtn<'a, Message>) -> Element<'a, Message, Theme, Renderer> {
        Element::new(from)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    is_pressed: bool,
}

impl State {
    /// Creates a new [`State`].
    ///
    /// [`State`]: struct.State.html
    pub fn new() -> State {
        State::default()
    }
}
