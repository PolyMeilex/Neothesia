use std::path::PathBuf;

use iced_native::{
    image, Align, Color, Column, Command, Container, Element, HorizontalAlignment, Image, Length,
    Program, Row, Text, VerticalAlignment,
};
use iced_wgpu::Renderer;

use crate::output_manager::OutputDescriptor;

use super::neo_btn::{self, NeoBtn};

enum Controls {
    SongSelect(SongSelectControls),
    Exit(ExitControls),
}
impl Controls {
    fn view(&mut self, carousel: &mut Carousel) -> Element<Message, Renderer> {
        match self {
            Controls::SongSelect(c) => c.view(carousel),
            Controls::Exit(c) => c.view(),
        }
    }
}

pub struct IcedMenu {
    midi_file: Option<lib_midi::Midi>,
    pub font_path: Option<PathBuf>,

    pub carousel: Carousel,

    controls: Controls,

    play_button: neo_btn::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileSelectPressed,
    FontSelectPressed,

    PrevPressed,
    NextPressed,
    PlayPressed,
    EscPressed,

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

            controls: Controls::SongSelect(SongSelectControls::new()),

            play_button: Default::default(),
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
                if self.carousel.check_next() {
                    self.carousel.next();
                }
            }
            Message::PrevPressed => {
                if self.carousel.check_prev() {
                    self.carousel.prev();
                }
            }

            Message::PlayPressed => {
                if self.midi_file.is_some() {
                    async fn play(m: Message) -> Message {
                        m
                    }

                    if self.midi_file.is_some() {
                        if let Some(midi) = std::mem::replace(&mut self.midi_file, None) {
                            if let Some(port) = self.carousel.get_item() {
                                let port = match port {
                                    #[cfg(feature = "synth")]
                                    OutputDescriptor::Synth(_) => OutputDescriptor::Synth(
                                        std::mem::replace(&mut self.font_path, None),
                                    ),
                                    _ => port.clone(),
                                };
                                let event = Message::MainMenuDone(midi, port);
                                return Command::from(play(event));
                            }
                        }
                    }
                }
            }

            Message::EscPressed => match self.controls {
                Controls::SongSelect(_) => {
                    self.controls = Controls::Exit(ExitControls::new());
                }
                Controls::Exit(_) => {
                    self.controls = Controls::SongSelect(SongSelectControls::new());
                }
            },

            Message::OutputsUpdated(outs) => {
                self.carousel.update(outs);
            }

            Message::MainMenuDone(_, _) => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let controls = self.controls.view(&mut self.carousel);

        let main: Element<_, _> = {
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
        } else {
            self.id = 0;
        }
    }

    fn prev(&mut self) {
        if self.check_prev() {
            self.id -= 1;
        } else {
            self.id = self.outputs.len() - 1;
        }
    }

    fn get_item(&self) -> Option<&OutputDescriptor> {
        self.outputs.get(self.id)
    }
}

#[derive(Default)]
struct SongSelectControls {
    file_select_button: neo_btn::State,
    synth_button: neo_btn::State,
    prev_button: neo_btn::State,
    next_button: neo_btn::State,
    play_button: neo_btn::State,
}

impl SongSelectControls {
    fn new() -> Self {
        Default::default()
    }
    fn view(&mut self, carousel: &mut Carousel) -> Element<Message, Renderer> {
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

        let item = carousel.get_item();

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
            .disabled(!carousel.check_prev())
            .on_press(Message::PrevPressed),
        );

        #[cfg(feature = "synth")]
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
            .disabled(!carousel.check_next())
            .on_press(Message::NextPressed),
        );

        let controls = Column::new()
            .align_items(Align::Center)
            .width(Length::Units(500))
            .height(Length::Units(250))
            .spacing(30)
            .push(file_select_button)
            .push(output)
            .push(select_row);

        Container::new(controls)
            .width(Length::Fill)
            .center_x()
            .into()
    }
}

#[derive(Default)]
struct ExitControls {
    no_button: neo_btn::State,
    yes_button: neo_btn::State,
}

impl ExitControls {
    fn new() -> Self {
        Self::default()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let output = Text::new("Do you want to exit?")
            .color(Color::WHITE)
            .size(30)
            .horizontal_alignment(HorizontalAlignment::Center)
            .vertical_alignment(VerticalAlignment::Center);

        let select_row = Row::new()
            .spacing(5)
            .height(Length::Units(50))
            .push(
                NeoBtn::new(
                    &mut self.no_button,
                    Text::new("No")
                        .size(30)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .on_press(Message::PrevPressed),
            )
            .push(
                NeoBtn::new(
                    &mut self.yes_button,
                    Text::new("Yes")
                        .size(30)
                        .horizontal_alignment(HorizontalAlignment::Center)
                        .vertical_alignment(VerticalAlignment::Center),
                )
                .width(Length::Fill)
                .on_press(Message::NextPressed),
            );

        let controls = Column::new()
            .align_items(Align::Center)
            .width(Length::Units(500))
            .spacing(30)
            .push(output)
            .push(select_row);

        Container::new(controls)
            .width(Length::Fill)
            .height(Length::Units(250))
            .center_x()
            .center_y()
            .into()
    }
}
