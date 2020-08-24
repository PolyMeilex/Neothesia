use iced_wgpu::Renderer;
use iced_winit::{
    button, slider, Align, Button, Color, Column, Command, Container, Element, HorizontalAlignment,
    Length, Program, Row, Slider, Text, VerticalAlignment,
};

pub struct Controls {
    text: String,
    increment_button: button::State,
    left_button: button::State,
    right_button: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    IncrementPressed,
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            text: "test".into(),
            increment_button: button::State::new(),
            left_button: button::State::new(),
            right_button: button::State::new(),
        }
    }

    // pub fn background_color(&self) -> Color {
    //     self.background_color
    // }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::IncrementPressed => {
                self.text = "2".into();
                // println!("test");
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        // let btn = Button::new(
        //     &mut self.increment_button,
        //     Text::new("Select File")
        //         .horizontal_alignment(HorizontalAlignment::Center)
        //         .vertical_alignment(VerticalAlignment::Center),
        // )
        // .width(Length::Fill)
        // .height(Length::Units(100))
        // .style(style::Button)
        // .on_press(Message::IncrementPressed);

        let btn = Row::new().height(Length::Units(100)).push(
            // Button::new(
            //     &mut self.increment_button,
            //     Text::new("Select File")
            //         .horizontal_alignment(HorizontalAlignment::Center)
            //         .vertical_alignment(VerticalAlignment::Center),
            // )
            // .width(Length::Fill)
            // .style(style::Button)
            // .on_press(Message::IncrementPressed),
            circle::Circle::new(
                &mut self.increment_button,
                Text::new("Select File")
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill),
        );

        let text = Text::new(format!("{:?}", self.text))
            .color(Color::WHITE)
            .height(Length::Units(100))
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center);

        let select_row = Row::new()
            .height(Length::Units(50))
            .push(
                Button::new(
                    &mut self.left_button,
                    Text::new("<")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .style(style::Button)
                .on_press(Message::IncrementPressed),
            )
            .push(
                Button::new(
                    &mut self.right_button,
                    Text::new(">")
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .style(style::Button)
                .on_press(Message::IncrementPressed),
            );

        // let row = Row::new().width(Length::Units(500)).push(btn);

        let coll = Column::new()
            .align_items(Align::Center)
            .width(Length::Units(500))
            .push(btn)
            .push(text)
            .push(select_row);

        let container = Container::new(coll)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y();

        container.into()
    }
}

mod style {
    use iced::{button, Background, Color, Vector};

    pub struct Button;

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(Color::from_rgba8(80, 80, 80, 0.6))),
                border_radius: 12,
                shadow_offset: Vector::new(1.0, 1.0),
                text_color: Color::from_rgb8(0xEE, 0xEE, 0xEE),
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(Color::from_rgba8(50, 50, 50, 0.6))),
                text_color: Color::WHITE,
                shadow_offset: Vector::new(1.0, 2.0),
                ..self.active()
            }
        }
    }
}

mod circle {
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
        button, layout, mouse, Background, Color, Element, Hasher, Layout, Length, Point, Size,
        Text, Vector, Widget,
    };

    pub struct Circle<'a, Message, B: Backend> {
        state: &'a mut button::State,
        width: Length,
        height: Length,
        padding: u16,
        border_radius: u16,
        content: Element<'a, Message, Renderer<B>>,
    }

    impl<'a, Message, B: Backend> Circle<'a, Message, B> {
        pub fn new<E>(state: &'a mut button::State, content: E) -> Self
        where
            E: Into<Element<'a, Message, Renderer<B>>>,
        {
            Self {
                state,
                width: Length::Shrink,
                height: Length::Shrink,
                padding: 5,
                border_radius: 10,
                content: content.into(),
            }
        }

        /// Sets the width of the [`Button`].
        ///
        /// [`Button`]: struct.Button.html
        pub fn width(mut self, width: Length) -> Self {
            self.width = width;
            self
        }

        /// Sets the height of the [`Button`].
        ///
        /// [`Button`]: struct.Button.html
        pub fn height(mut self, height: Length) -> Self {
            self.height = height;
            self
        }
    }

    // impl<'a, Message, B> Widget<Message, Renderer<B>> for Circle<'a, Message, B>
    // where
    //     Message: Clone,
    //     B: Backend,
    // {
    impl<'a, Message, B> Widget<Message, Renderer<B>> for Circle<'a, Message, B>
    where
        B: Backend,
    {
        fn width(&self) -> Length {
            Length::Shrink
        }

        fn height(&self) -> Length {
            Length::Shrink
        }

        fn layout(&self, renderer: &Renderer<B>, limits: &layout::Limits) -> layout::Node {
            let padding = f32::from(self.padding);
            let limits = limits
                // .min_width(self.min_width)
                // .min_height(self.min_height)
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

        fn draw(
            &self,
            renderer: &mut Renderer<B>,
            _defaults: &Defaults,
            layout: Layout<'_>,
            cursor_position: Point,
        ) -> (Primitive, mouse::Interaction) {
            let bounds = layout.bounds();

            let (content, _) = self.content.draw(
                renderer,
                &Defaults {
                    text: defaults::Text {
                        color: Color::WHITE,
                    },
                },
                layout,
                cursor_position,
            );

            (
                Primitive::Group {
                    primitives: vec![
                        Primitive::Clip {
                            bounds: Rectangle {
                                y: bounds.y,
                                height: bounds.height - self.border_radius as f32,
                                ..bounds
                            },
                            offset: Vector::new(0, 0),
                            content: Box::new(Primitive::Quad {
                                bounds: Rectangle {
                                    y: bounds.y,
                                    ..bounds
                                },
                                // background: Background::Color(Color::BLACK),
                                background: Background::Color(
                                    [17.0 / 255.0, 17.0 / 255.0, 17.0 / 255.0, 0.6].into(),
                                    // [1.0, 0.0, 0.0, 0.8].into(),
                                ),
                                border_radius: self.border_radius,
                                border_width: 0,
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
                                // background: Background::Color(Color::BLACK),
                                background: Background::Color(
                                    [160.0 / 255.0, 81.0 / 255.0, 1.0, 1.0].into(),
                                ),
                                border_radius: self.border_radius,
                                border_width: 0,
                                border_color: Color::TRANSPARENT,
                            }),
                        },
                        content,
                    ],
                },
                mouse::Interaction::default(),
            )
        }
    }

    impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for Circle<'a, Message, B>
    where
        B: 'a + Backend,
        Message: 'a,
    {
        fn into(self) -> Element<'a, Message, Renderer<B>> {
            Element::new(self)
        }
    }
}

use circle::Circle;
