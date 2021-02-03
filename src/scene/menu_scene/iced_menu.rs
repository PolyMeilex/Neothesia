use std::path::PathBuf;

use iced_native::{
    image, Align, Color, Column, Command, Container, Element, HorizontalAlignment, Image, Length,
    Program, Row, Text, VerticalAlignment,
};
use iced_wgpu::Renderer;

use crate::main_state::MainState;
use crate::output_manager::OutputDescriptor;

use super::neo_btn::{self, NeoBtn};

enum Controls {
    SongSelect(SongSelectControls),
    Exit(ExitControls),
}

pub struct IcedMenu {
    pub play_along: bool,

    midi_file: bool,
    font_path: Option<PathBuf>,

    pub carousel: Carousel,

    controls: Controls,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileSelectPressed,

    FontSelectPressed,

    PrevPressed,
    NextPressed,

    #[cfg(feature = "play_along")]
    TogglePlayAlong(bool),

    EnterPressed,
    EscPressed,

    MidiFileUpdate(bool),
    OutputsUpdated(Vec<OutputDescriptor>),

    // Output
    OutputFileSelected(PathBuf),
    OutputMainMenuDone(OutputDescriptor),
    OutputAppExit,
}

impl IcedMenu {
    pub fn new(state: &mut MainState) -> Self {
        let mut carousel = Carousel::new();
        let outputs = state.output_manager.get_outputs();
        carousel.update(outputs);

        let out_id = state.output_manager.selected_output_id;
        if let Some(id) = out_id {
            carousel.id = id;
        }

        Self {
            #[cfg(feature = "play_along")]
            play_along: state.config.play_along,
            #[cfg(not(feature = "play_along"))]
            play_along: false,

            midi_file: state.midi_file.is_some(),
            font_path: state.output_manager.selected_font_path.clone(),

            carousel,

            controls: Controls::SongSelect(SongSelectControls::new()),
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

                        return Command::from(async { Message::OutputFileSelected(path) });
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
            #[cfg(feature = "play_along")]
            Message::TogglePlayAlong(is) => {
                self.play_along = is;
            }

            Message::EnterPressed => match self.controls {
                Controls::SongSelect(_) => {
                    if self.midi_file {
                        async fn play(m: Message) -> Message {
                            m
                        }

                        if let Some(port) = self.carousel.get_item() {
                            let port = match port {
                                #[cfg(feature = "synth")]
                                OutputDescriptor::Synth(_) => OutputDescriptor::Synth(
                                    std::mem::replace(&mut self.font_path, None),
                                ),
                                _ => port.clone(),
                            };
                            let event = Message::OutputMainMenuDone(port);
                            return Command::from(play(event));
                        }
                    }
                }
                Controls::Exit(_) => {
                    return Command::from(async { Message::OutputAppExit });
                }
            },

            Message::EscPressed => match self.controls {
                Controls::SongSelect(_) => {
                    self.controls = Controls::Exit(ExitControls::new());
                }
                Controls::Exit(_) => {
                    self.controls = Controls::SongSelect(SongSelectControls::new());
                }
            },

            Message::MidiFileUpdate(is) => self.midi_file = is,

            Message::OutputsUpdated(outs) => {
                self.carousel.update(outs);
            }

            Message::OutputFileSelected(_) => {}
            Message::OutputMainMenuDone(_) => {}
            Message::OutputAppExit => {}
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let (controls, footer) = match &mut self.controls {
            Controls::SongSelect(c) => {
                let (content, footer) = c.view(&mut self.carousel, self.midi_file, self.play_along);
                (content, Some(footer))
            }
            Controls::Exit(c) => (c.view(), None),
        };

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

        let mut out = Column::new().push(main);

        if let Some(footer) = footer {
            out = out.push(footer);
        }

        out.into()
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
    fn view(
        &mut self,
        carousel: &mut Carousel,
        midi_file: bool,
        play_along: bool,
    ) -> (Element<Message, Renderer>, Element<Message, Renderer>) {
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
            .unwrap_or_else(|| "Disconected".to_string());

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

        (
            Container::new(controls)
                .width(Length::Fill)
                .center_x()
                .into(),
            Self::footer(&mut self.play_button, &carousel, midi_file, play_along),
        )
    }

    #[allow(unused_variables)]
    fn footer<'a>(
        play_button: &'a mut neo_btn::State,
        carousel: &Carousel,
        midi_file: bool,
        play_along: bool,
    ) -> Element<'a, Message, Renderer> {
        let content: Element<Message, Renderer> = if midi_file && carousel.get_item().is_some() {
            let btn = NeoBtn::new(
                play_button,
                Text::new("Play")
                    .size(30)
                    .horizontal_alignment(HorizontalAlignment::Center)
                    .vertical_alignment(VerticalAlignment::Center)
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
                use iced_native::Checkbox;
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
            .align_x(Align::End)
            .align_y(Align::End)
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
                .on_press(Message::EscPressed),
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
                .on_press(Message::EnterPressed),
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

pub struct CheckboxStyle;

const SURFACE: Color = Color::from_rgb(
    0x30 as f32 / 255.0,
    0x34 as f32 / 255.0,
    0x3B as f32 / 255.0,
);

impl iced_graphics::checkbox::StyleSheet for CheckboxStyle {
    fn active(&self, is_checked: bool) -> iced_graphics::checkbox::Style {
        let active = Color::from_rgba8(160, 81, 255, 1.0);
        iced_graphics::checkbox::Style {
            background: if is_checked { active } else { SURFACE }.into(),
            checkmark_color: Color::WHITE,
            border_radius: 2.0,
            border_width: 1.0,
            border_color: active,
        }
    }

    fn hovered(&self, is_checked: bool) -> iced_graphics::checkbox::Style {
        let active = Color::from_rgba8(160, 81, 255, 1.0);
        iced_graphics::checkbox::Style {
            background: Color {
                a: 0.8,
                ..if is_checked { active } else { SURFACE }
            }
            .into(),
            ..self.active(is_checked)
        }
    }
}
