use std::path::PathBuf;

use crate::target::Target;
use iced_native::widget::{Column, Container, Image, Row, Text};
use iced_native::{
    alignment::Horizontal, alignment::Vertical, command::Action, image, Alignment, Color, Command,
    Element, Length, Program,
};
use iced_wgpu::Renderer;
use midir::MidiInputPort;

use crate::output_manager::{InputDescriptior, OutputDescriptor};

use super::neo_btn::{self, NeoBtn};

mod carousel;
use carousel::Carousel;

mod song_select;
use song_select::SongSelectControls;

#[cfg(feature = "play_along")]
mod input_select;
#[cfg(feature = "play_along")]
use input_select::InputSelectControls;

enum Controls {
    SongSelect(SongSelectControls),
    #[cfg(feature = "play_along")]
    InputSelect(InputSelectControls),
    Exit(ExitControls),
}

pub struct IcedMenu {
    pub play_along: bool,

    midi_file: bool,
    font_path: Option<PathBuf>,

    pub out_carousel: Carousel<OutputDescriptor>,
    pub in_carousel: Carousel<MidiInputPort>,

    controls: Controls,

    logo_handle: image::Handle,
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
    OutputMainMenuDone((OutputDescriptor, InputDescriptior)),
    OutputAppExit,
}

impl IcedMenu {
    pub fn new(target: &mut Target) -> Self {
        let mut out_carousel = Carousel::new();

        let output_manager = target.output_manager.borrow();

        let outputs = output_manager.get_outputs();
        out_carousel.update(outputs);

        let out_id = output_manager.selected_output_id;
        if let Some(id) = out_id {
            out_carousel.select(id);
        }

        let in_carousel = {
            let midi_in = midir::MidiInput::new("Neothesia-in").unwrap();
            let ports: Vec<_> = midi_in.ports().into_iter().collect();

            let mut in_carousel = Carousel::new();
            in_carousel.update(ports);
            in_carousel
        };

        Self {
            #[cfg(feature = "play_along")]
            play_along: target.state.config.play_along,
            #[cfg(not(feature = "play_along"))]
            play_along: false,

            midi_file: target.state.midi_file.is_some(),
            font_path: output_manager.selected_font_path.clone(),

            out_carousel,
            in_carousel,

            controls: Controls::SongSelect(SongSelectControls::new()),

            logo_handle: image::Handle::from_memory(include_bytes!("./img/banner.png").to_vec()),
        }
    }
}

impl Program for IcedMenu {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::FileSelectPressed => {
                match rfd::FileDialog::new()
                    .add_filter("midi", &["mid", "midi"])
                    .pick_file()
                {
                    Some(path) => {
                        log::info!("File path = {:?}", path);

                        return Command::perform(async { path }, Message::OutputFileSelected);
                    }
                    _ => {
                        log::warn!("User canceled dialog");
                    }
                }
            }

            Message::FontSelectPressed => {
                match rfd::FileDialog::new()
                    .add_filter("SoundFont2", &["sf2"])
                    .pick_file()
                {
                    Some(path) => {
                        log::info!("Font path = {:?}", path);
                        self.font_path = Some(path);
                    }
                    _ => {
                        log::warn!("User canceled dialog");
                    }
                }
            }

            Message::NextPressed => match self.controls {
                Controls::SongSelect(_) => {
                    if self.out_carousel.check_next() {
                        self.out_carousel.next();
                    }
                }
                #[cfg(feature = "play_along")]
                Controls::InputSelect(_) => {
                    if self.in_carousel.check_next() {
                        self.in_carousel.next();
                    }
                }
                _ => {}
            },
            Message::PrevPressed => match self.controls {
                Controls::SongSelect(_) => {
                    if self.out_carousel.check_prev() {
                        self.out_carousel.prev();
                    }
                }
                #[cfg(feature = "play_along")]
                Controls::InputSelect(_) => {
                    if self.in_carousel.check_prev() {
                        self.in_carousel.prev();
                    }
                }
                _ => {}
            },
            #[cfg(feature = "play_along")]
            Message::TogglePlayAlong(is) => {
                self.play_along = is;
            }

            Message::EnterPressed => match self.controls {
                Controls::SongSelect(_) => {
                    if self.midi_file {
                        if let Some(port) = self.out_carousel.get_item() {
                            if self.play_along {
                                #[cfg(feature = "play_along")]
                                {
                                    self.controls =
                                        Controls::InputSelect(InputSelectControls::new())
                                }
                            } else {
                                let port = match port {
                                    #[cfg(feature = "synth")]
                                    OutputDescriptor::Synth(_) => OutputDescriptor::Synth(
                                        std::mem::replace(&mut self.font_path, None),
                                    ),
                                    _ => port.clone(),
                                };

                                // TODO: Dumb input
                                let in_port = self.in_carousel.get_item().unwrap().clone();
                                let in_port = InputDescriptior { input: in_port };

                                return Command::perform(
                                    async { (port, in_port) },
                                    Message::OutputMainMenuDone,
                                );
                            }
                        }
                    }
                }
                #[cfg(feature = "play_along")]
                Controls::InputSelect(_) => {
                    if self.midi_file {
                        if let Some(port) = self.out_carousel.get_item() {
                            let port = match port {
                                #[cfg(feature = "synth")]
                                OutputDescriptor::Synth(_) => OutputDescriptor::Synth(
                                    std::mem::replace(&mut self.font_path, None),
                                ),
                                _ => port.clone(),
                            };
                            // TODO: Dumb input
                            let in_port = self.in_carousel.get_item().unwrap().clone();
                            let in_port = InputDescriptior { input: in_port };

                            return Command::perform(
                                async { (port, in_port) },
                                Message::OutputMainMenuDone,
                            );
                        }
                    }
                }
                Controls::Exit(_) => {
                    return Command::single(Action::Future(Box::pin(async {
                        Message::OutputAppExit
                    })));
                }
            },

            Message::EscPressed => match self.controls {
                Controls::SongSelect(_) => {
                    self.controls = Controls::Exit(ExitControls::new());
                }
                #[cfg(feature = "play_along")]
                Controls::InputSelect(_) => {
                    self.controls = Controls::SongSelect(SongSelectControls::new());
                }
                Controls::Exit(_) => {
                    self.controls = Controls::SongSelect(SongSelectControls::new());
                }
            },

            Message::MidiFileUpdate(is) => self.midi_file = is,

            Message::OutputsUpdated(outs) => {
                self.out_carousel.update(outs);
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
                let (content, footer) =
                    c.view(&mut self.out_carousel, self.midi_file, self.play_along);
                (content, Some(footer))
            }
            #[cfg(feature = "play_along")]
            Controls::InputSelect(c) => {
                let (content, footer) =
                    c.view(&mut self.in_carousel, self.midi_file, self.play_along);
                (content, Some(footer))
            }
            Controls::Exit(c) => (c.view(), None),
        };

        let main: Element<_, _> = {
            let image = Image::new(self.logo_handle.clone());
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
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let select_row = Row::new()
            .spacing(5)
            .height(Length::Units(50))
            .push(
                NeoBtn::new(
                    &mut self.no_button,
                    Text::new("No")
                        .size(30)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .width(Length::Fill)
                .on_press(Message::EscPressed),
            )
            .push(
                NeoBtn::new(
                    &mut self.yes_button,
                    Text::new("Yes")
                        .size(30)
                        .horizontal_alignment(Horizontal::Center)
                        .vertical_alignment(Vertical::Center),
                )
                .width(Length::Fill)
                .on_press(Message::EnterPressed),
            );

        let controls = Column::new()
            .align_items(Alignment::Center)
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
