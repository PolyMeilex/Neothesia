use iced_native::{
    image, Align, Color, Column, Command, Container, Element, HorizontalAlignment, Image, Length,
    Program, Row, Text, VerticalAlignment,
};
use iced_wgpu::Renderer;

use crate::midi_device::{MidiDevicesManager, MidiPortInfo};

use std::sync::Arc;
pub struct IcedMenu {
    midi_device_menager: MidiDevicesManager,
    selected_out_id: Option<usize>,
    midi_file: Option<Arc<lib_midi::Midi>>,

    file_select_button: neo_btn::State,
    prev_button: neo_btn::State,
    next_button: neo_btn::State,
    play_button: neo_btn::State,

    prev_out_exists: bool,
    next_out_exists: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateOuts,
    UpdatePrevNext,

    FileSelectPressed,
    PrevPressed,
    NextPressed,
    PlayPressed,

    MainMenuDone(Arc<lib_midi::Midi>, Option<MidiPortInfo>),
}

impl IcedMenu {
    pub fn new(midi_file: Option<Arc<lib_midi::Midi>>) -> Self {
        Self {
            midi_device_menager: MidiDevicesManager::new(),
            selected_out_id: None,
            midi_file,

            file_select_button: neo_btn::State::new(),
            prev_button: neo_btn::State::new(),
            next_button: neo_btn::State::new(),
            play_button: neo_btn::State::new(),

            prev_out_exists: false,
            next_out_exists: false,
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
                            Some(Arc::new(midi))
                        } else {
                            None
                        }
                    }
                    _ => {
                        log::error!("User canceled dialog");
                    }
                }
            }
            // Next / Prev Logic
            Message::UpdateOuts => {
                self.midi_device_menager.update_outs();

                self.selected_out_id = if let Some(id) = self.selected_out_id {
                    // Check is selected out still exists
                    let out_id = self.midi_device_menager.check_out_id(id);

                    if out_id.is_some() {
                        out_id
                    } else {
                        // try to reset selection to 0 id
                        self.midi_device_menager.check_out_id(0)
                    }
                } else {
                    // try to reset selection to 0 id
                    self.midi_device_menager.check_out_id(0)
                };

                self.update(Message::UpdatePrevNext);
            }
            Message::UpdatePrevNext => {
                self.next_out_exists = if let Some(id) = self.selected_out_id {
                    self.midi_device_menager.check_out_id(id + 1).is_some()
                } else {
                    false
                };

                self.prev_out_exists = if let Some(id) = self.selected_out_id {
                    if id > 0 {
                        self.midi_device_menager.check_out_id(id - 1).is_some()
                    } else {
                        false
                    }
                } else {
                    false
                };
            }
            Message::NextPressed => {
                if let Some(id) = self.selected_out_id {
                    if let Some(id) = self.midi_device_menager.check_out_id(id + 1) {
                        self.selected_out_id = Some(id);
                    }
                }
                self.update(Message::UpdatePrevNext);
            }
            Message::PrevPressed => {
                if let Some(id) = self.selected_out_id {
                    // Make sure to not triger subtract with overflow
                    if id > 0 {
                        if let Some(id) = self.midi_device_menager.check_out_id(id - 1) {
                            self.selected_out_id = Some(id);
                        }
                    }
                }
                self.update(Message::UpdatePrevNext);
            }
            //
            Message::PlayPressed => {
                if self.midi_file.is_some() {
                    async fn play(
                        file: Arc<lib_midi::Midi>,
                        port: Option<MidiPortInfo>,
                    ) -> Message {
                        Message::MainMenuDone(file, port)
                    }

                    let port = if let Some(id) = self.selected_out_id {
                        if let Some(out) = self.midi_device_menager.get_out(id) {
                            Some(out.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let midi =
                        std::mem::replace(&mut self.midi_file, None).expect("No midi file!!");
                    return Command::from(play(midi, port));
                }
            }
            Message::MainMenuDone(_, _) => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        self.update(Message::UpdateOuts);

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

            let selected_out = if let Some(id) = self.selected_out_id {
                self.midi_device_menager.get_out(id)
            } else {
                None
            };

            let text = if let Some(out) = selected_out {
                out.name.clone()
            } else {
                "No Midi Devices".into()
            };

            let text = Text::new(text)
                .color(Color::WHITE)
                // .height(Length::Units(100))
                .size(30)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center);

            let select_row = Row::new()
                .height(Length::Units(50))
                .push(
                    NeoBtn::new(
                        &mut self.prev_button,
                        Text::new("<")
                            .size(40)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .vertical_alignment(VerticalAlignment::Center),
                    )
                    .width(Length::Fill)
                    .disabled(!self.prev_out_exists)
                    .on_press(Message::PrevPressed),
                )
                .push(
                    NeoBtn::new(
                        &mut self.next_button,
                        Text::new(">")
                            .size(40)
                            .horizontal_alignment(HorizontalAlignment::Center)
                            .vertical_alignment(VerticalAlignment::Center),
                    )
                    .width(Length::Fill)
                    .disabled(!self.next_out_exists)
                    .on_press(Message::NextPressed),
                );

            let controls = Column::new()
                .align_items(Align::Center)
                .width(Length::Units(500))
                .spacing(30)
                .push(file_select_button)
                .push(text)
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
            let content: Element<Self::Message, Self::Renderer> = if self.midi_file.is_some() {
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
                .padding(10)
                .width(Length::Fill)
                .height(Length::Units(70))
                .align_x(Align::End)
                .align_y(Align::End);
            footer.into()
        };

        Column::new().push(main).push(footer).into()
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
        border_radius: u16,
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
                border_radius: 7,
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

        pub fn min_width(mut self, min_width: u32) -> Self {
            self.min_width = min_width;
            self
        }

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
                                height: bounds.height - self.border_radius as f32,
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
                                background: Background::Color(colors.1),
                                border_radius: self.border_radius,
                                border_width: 0,
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
