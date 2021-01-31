// For now, to implement a custom native widget you will need to add
// `iced_native` and `iced_wgpu` to your dependencies.
//
// Then, you simply need to define your widget type and implement the
// `iced_native::Widget` trait with the `iced_wgpu::Renderer`.
//
// Of course, you can choose to make the implementation renderer-agnostic,
// if you wish to, by creating your own `Renderer` trait, which could be
// implemented by `iced_wgpu` and other renderers.
use iced_graphics::{defaults, Backend, Defaults, Primitive, Rectangle, Renderer};
use iced_native::{
    layout, mouse, Background, Clipboard, Color, Element, Event, Hasher, Layout, Length, Point,
    Vector, Widget,
};

pub struct NeoBtn<'a, Message: Clone, B: Backend> {
    state: &'a mut State,
    width: Length,
    height: Length,
    min_width: u32,
    min_height: u32,
    padding: u16,
    border_radius: f32,
    disabled: bool,
    content: Element<'a, Message, Renderer<B>>,
    on_press: Option<Message>,
}

impl<'a, Message: Clone, B: Backend> NeoBtn<'a, Message, B> {
    pub fn new<E>(state: &'a mut State, content: E) -> Self
    where
        E: Into<Element<'a, Message, Renderer<B>>>,
    {
        Self {
            state,
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            min_width: 0,
            min_height: 0,
            padding: 5,
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

    // pub fn min_width(mut self, min_width: u32) -> Self {
    //     self.min_width = min_width;
    //     self
    // }

    pub fn min_height(mut self, min_height: u32) -> Self {
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

// impl<'a, Message, B> Widget<Message, Renderer<B>> for Circle<'a, Message, B>
// where
//     Message: Clone,
//     B: Backend,
// {
impl<'a, Message: Clone, B> Widget<Message, Renderer<B>> for NeoBtn<'a, Message, B>
where
    B: Backend,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer<B>, limits: &layout::Limits) -> layout::Node {
        let padding = f32::from(self.padding);
        let limits = limits
            .min_width(self.min_width)
            .min_height(self.min_height)
            .width(self.width)
            .height(self.height)
            .pad(padding);

        let mut content = self.content.layout(renderer, &limits);
        content.move_to(Point::new(padding, padding));

        let size = limits.resolve(content.size()).pad(padding);

        layout::Node::with_children(size, vec![content])
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.width.hash(state);
        self.content.hash_layout(state);
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        _renderer: &Renderer<B>,
        _clipboard: Option<&dyn Clipboard>,
    ) -> iced_native::event::Status {
        if self.disabled {
            return iced_native::event::Status::Ignored;
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    self.state.is_pressed = bounds.contains(cursor_position);
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(on_press) = self.on_press.clone() {
                    let bounds = layout.bounds();

                    let is_clicked = self.state.is_pressed && bounds.contains(cursor_position);

                    self.state.is_pressed = false;

                    if is_clicked {
                        messages.push(on_press);
                    }
                }
            }
            _ => {}
        };

        iced_native::event::Status::Ignored
    }

    fn draw(
        &self,
        renderer: &mut Renderer<B>,
        _defaults: &Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> <Renderer<B> as iced_native::Renderer>::Output {
        let bounds = layout.bounds();
        let is_mouse_over = bounds.contains(cursor_position);

        let (content, _) = self.content.draw(
            renderer,
            &Defaults {
                text: defaults::Text {
                    color: if self.disabled {
                        Color::new(0.3, 0.3, 0.3, 1.0)
                    } else {
                        Color::WHITE
                    },
                },
            },
            layout,
            cursor_position,
            viewport,
        );

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

        (
            Primitive::Group {
                primitives: vec![
                    Primitive::Clip {
                        bounds: Rectangle {
                            y: bounds.y,
                            height: bounds.height - self.border_radius,
                            ..bounds
                        },
                        offset: Vector::new(0, 0),
                        content: Box::new(Primitive::Quad {
                            bounds: Rectangle {
                                y: bounds.y,
                                ..bounds
                            },
                            background: Background::Color(colors.0),
                            border_radius: self.border_radius,
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        }),
                    },
                    Primitive::Clip {
                        bounds: Rectangle {
                            y: bounds.y + bounds.height - self.border_radius as f32,
                            height: self.border_radius as f32,
                            ..bounds
                        },
                        offset: Vector::new(0, 0),
                        content: Box::new(Primitive::Quad {
                            bounds: Rectangle {
                                y: bounds.y,
                                ..bounds
                            },
                            background: Background::Color(colors.1),
                            border_radius: self.border_radius,
                            border_width: 0.0,
                            border_color: Color::TRANSPARENT,
                        }),
                    },
                    content,
                ],
            },
            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            },
        )
    }
}

impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for NeoBtn<'a, Message, B>
where
    B: 'a + Backend,
    Message: 'a + Clone,
{
    fn into(self) -> Element<'a, Message, Renderer<B>> {
        Element::new(self)
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
    #[allow(dead_code)]
    pub fn new() -> State {
        State::default()
    }
}
