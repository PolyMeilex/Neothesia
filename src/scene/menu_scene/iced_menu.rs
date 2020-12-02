use std::path::PathBuf;

use iced_native::{
    image, Align, Color, Column, Command, Container, Element, HorizontalAlignment, Image, Length,
    Program, Row, Text, VerticalAlignment,
};
use iced_wgpu::Renderer;

use crate::output_manager::OutputDescriptor;

pub struct IcedMenu {
    midi_file: Option<lib_midi::Midi>,
    pub font_path: Option<PathBuf>,

    pub carousel: Carousel,

    file_select_button: neo_btn::State,
    synth_button: neo_btn::State,
    prev_button: neo_btn::State,
    next_button: neo_btn::State,
    play_button: neo_btn::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileSelectPressed,
    FontSelectPressed,

    PrevPressed,
    NextPressed,
    PlayPressed,

    OutputsUpdated(Vec<OutputDescriptor>),

    MainMenuDone(lib_midi::Midi, OutputDescriptor),
}

impl IcedMenu {
    pub fn new(
        midi_file: Option<lib_midi::Midi>,
        outputs: Vec<OutputDescriptor>,
        out_id: Option<usize>,
        font_path: Option<PathBuf>,
    ) -> Self {
        let mut carousel = Carousel::new();
        carousel.update(outputs);

        if let Some(id) = out_id {
            carousel.id = id;
        }

        Self {
            midi_file,
            font_path,

            carousel,

            file_select_button: neo_btn::State::new(),
            synth_button: neo_btn::State::new(),
            prev_button: neo_btn::State::new(),
            next_button: neo_btn::State::new(),
            play_button: neo_btn::State::new(),
        }
    }
}

impl Program for IcedMenu {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileSelectPressed => {
                use nfd2::Response;

                match nfd2::DialogBuilder::single()
                    .filter("mid,midi")
                    .open()
                    .expect("File Dialog Error")
                {
                    Response::Okay(path) => {
                        log::info!("File path = {:?}", path);
                        let midi = lib_midi::Midi::new(path.to_str().unwrap());

                        if let Err(e) = &midi {
                            log::error!("{}", e);
                        }

                        self.midi_file = if let Ok(midi) = midi {
                            Some(midi)
                        } else {
                            None
                        };
                    }
                    _ => {
                        log::error!("User canceled dialog");
                    }
                }
            }

            Message::FontSelectPressed => {
                use nfd2::Response;

                match nfd2::DialogBuilder::single()
                    .filter("sf2")
                    .open()
                    .expect("Font Dialog Error")
                {
                    Response::Okay(path) => {
                        log::info!("Font path = {:?}", path);
                        self.font_path = Some(path);
                    }
                    _ => {
                        log::error!("User canceled dialog");
                    }
                }
            }

            Message::NextPressed => {
                self.carousel.next();
            }
            Message::PrevPressed => {
                self.carousel.prev();
            }

            Message::PlayPressed => {
                if self.midi_file.is_some() {
                    async fn play(m: Message) -> Message {
                        m
                    }

                    if self.midi_file.is_some() {
                        if let Some(midi) = std::mem::replace(&mut self.midi_file, None) {
                            if let Some(port) = self.carousel.get_item() {
                                let port = if let OutputDescriptor::Synth(_) = port {
                                    OutputDescriptor::Synth(std::mem::replace(
                                        &mut self.font_path,
                                        None,
                                    ))
                                } else {
                                    port.clone()
                                };
                                let event = Message::MainMenuDone(midi, port);
                                return Command::from(play(event));
                            }
                        }
                    }
                }
            }

            Message::OutputsUpdated(outs) => {
                self.carousel.update(outs);
            }

            Message::MainMenuDone(_, _) => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let main: Element<_, _> = {
            let file_select_button = Row::new().height(Length::Units(100)).push(
                NeoBtn::new(
                    &mut self.file_select_button,
                    Text::new("Select File")
                        .size(40)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .on_press(Message::FileSelectPressed),
            );

            let item = self.carousel.get_item();

            let label = item
                .map(|o| o.to_string())
                .unwrap_or("Disconected".to_string());

            let output = Text::new(label)
                .color(Color::WHITE)
                .size(30)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center);

            let mut select_row = Row::new().height(Length::Units(50)).push(
                NeoBtn::new(
                    &mut self.prev_button,
                    Text::new("<")
                        .size(40)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .disabled(!self.carousel.check_prev())
                .on_press(Message::PrevPressed),
            );

            if let Some(OutputDescriptor::Synth(_)) = item {
                select_row = select_row.push(
                    NeoBtn::new(
                        &mut self.synth_button,
                        Text::new("Soundfont")
                            .size(20)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .vertical_alignment(VerticalAlignment::Center),
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
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .disabled(!self.carousel.check_next())
                .on_press(Message::NextPressed),
            );

            let controls = Column::new()
                .align_items(Align::Center)
                .width(Length::Units(500))
                .spacing(30)
                .push(file_select_button)
                .push(output)
                .push(select_row);

            let controls = Container::new(controls).width(Length::Fill).center_x();

            let image = Image::new(image::Handle::from_memory(
                include_bytes!("./img/baner.png").to_vec(),
            ));
            let image = Container::new(image)
                .center_x()
                .center_y()
                .width(Length::Fill);

            let main = Column::new()
                .width(Length::Fill)
                .spacing(40)
                .max_width(650)
                .push(image)
                .push(controls);

            let centered_main = Container::new(main)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y();

            centered_main.into()
        };

        let footer: Element<_, _> = {
            let content: Element<Self::Message, Self::Renderer> =
                if self.midi_file.is_some() && self.carousel.get_item().is_some() {
                    let btn = NeoBtn::new(
                        &mut self.play_button,
                        Text::new("Play")
                            .size(30)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .vertical_alignment(VerticalAlignment::Center)
                            .color(Color::WHITE),
                    )
                    .min_height(50)
                    .height(Length::Fill)
                    .width(Length::Units(150))
                    .on_press(Message::PlayPressed);

                    btn.into()
                } else {
                    Row::new().into()
                };

            let footer = Container::new(content)
                .width(Length::Fill)
                .height(Length::Units(70))
                .align_x(Align::End)
                .align_y(Align::End);
            footer.into()
        };

        Column::new().push(main).push(footer).into()
    }
}

pub struct Carousel {
    outputs: Vec<OutputDescriptor>,
    id: usize,
}

impl Carousel {
    fn new() -> Self {
        Self {
            outputs: Vec::new(),
            id: 0,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    fn update(&mut self, outs: Vec<OutputDescriptor>) {
        self.outputs = outs;
    }

    fn check_next(&self) -> bool {
        self.id < self.outputs.len() - 1
    }

    fn check_prev(&self) -> bool {
        self.id > 0
    }

    fn next(&mut self) {
        if self.check_next() {
            self.id += 1;
        }
    }

    fn prev(&mut self) {
        if self.check_prev() {
            self.id -= 1;
        }
    }

    fn get_item(&self) -> Option<&OutputDescriptor> {
        self.outputs.get(self.id)
    }
}

mod neo_btn {
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
        pub fn new() -> State {
            State::default()
        }
    }
}

use neo_btn::NeoBtn;
