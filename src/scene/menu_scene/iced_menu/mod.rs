use std::path::PathBuf;

use crate::target::Target;
use crate::NeothesiaEvent;
use iced_native::widget::helpers::{column, container, image, row, text};
use iced_native::{
    alignment::Horizontal, alignment::Vertical, image, Alignment, Color, Element, Length,
};
use iced_wgpu::Renderer;

use crate::output_manager::OutputDescriptor;

use super::neo_btn::{self, NeoBtn};
use crate::ui::iced_state::Program;

mod carousel;
use carousel::Carousel;

mod song_select;
use song_select::SongSelectControls;

mod input_select;
use input_select::InputSelectControls;

enum Controls {
    SongSelect(SongSelectControls),
    InputSelect(InputSelectControls),
    Exit(ExitControls),
}

pub struct IcedMenu {
    play_along: bool,

    midi_file: bool,
    font_path: Option<PathBuf>,

    out_carousel: Carousel<OutputDescriptor>,
    in_carousel: Carousel<midi_io::MidiInputPort>,

    controls: Controls,

    logo_handle: image::Handle,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileSelectPressed,

    FontSelectPressed,

    PrevPressed,
    NextPressed,

    #[allow(unused)]
    TogglePlayAlong(bool),

    ContinuePressed,
    BackPressed,

    OutputsUpdated(Vec<OutputDescriptor>),
}

impl IcedMenu {
    pub fn new(target: &mut Target) -> Self {
        let mut out_carousel = Carousel::new();

        let outputs = target.output_manager.get_outputs();

        let config_id = outputs
            .iter()
            .position(|o| o.to_string() == target.config.output);

        out_carousel.update(outputs);

        let out_id = target.output_manager.selected_output_id.or(config_id);
        if let Some(id) = out_id {
            out_carousel.select(id);
        }

        let mut in_carousel = Carousel::new();
        in_carousel.update(target.input_manager.inputs());

        Self {
            play_along: target.config.play_along,

            midi_file: target.midi_file.is_some(),
            font_path: target.output_manager.selected_font_path.clone(),

            out_carousel,
            in_carousel,

            controls: Controls::SongSelect(SongSelectControls::new()),

            logo_handle: image::Handle::from_memory(include_bytes!("../img/banner.png").to_vec()),
        }
    }
}

impl Program for IcedMenu {
    type Message = Message;

    fn update(&mut self, target: &mut Target, message: Message) {
        match message {
            Message::FileSelectPressed => {
                match rfd::FileDialog::new()
                    .add_filter("midi", &["mid", "midi"])
                    .pick_file()
                {
                    Some(path) => {
                        log::info!("File path = {:?}", path);

                        let midi = lib_midi::Midi::new(path.to_str().unwrap());

                        if let Err(e) = &midi {
                            log::error!("{}", e);
                        }

                        target.midi_file = midi.ok();
                        self.midi_file = target.midi_file.is_some();
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
                Controls::InputSelect(_) => {
                    if self.in_carousel.check_prev() {
                        self.in_carousel.prev();
                    }
                }
                _ => {}
            },

            Message::TogglePlayAlong(is) => {
                self.play_along = is;
            }

            Message::ContinuePressed => match self.controls {
                Controls::SongSelect(_) => {
                    if self.midi_file {
                        if let Some(port) = self.out_carousel.get_item() {
                            target.config.play_along = self.play_along;

                            let port = match port {
                                #[cfg(feature = "synth")]
                                OutputDescriptor::Synth(_) => OutputDescriptor::Synth(
                                    std::mem::replace(&mut self.font_path, None),
                                ),
                                _ => port.clone(),
                            };

                            target.output_manager.selected_output_id = Some(self.out_carousel.id());
                            target.output_manager.connect(port);

                            target.config.output =
                                format!("{}", target.output_manager.current_output());

                            if self.play_along {
                                self.controls = Controls::InputSelect(InputSelectControls::new())
                            } else {
                                target
                                    .proxy
                                    .send_event(NeothesiaEvent::MainMenu(super::Event::Play))
                                    .unwrap();
                            }
                        }
                    }
                }

                Controls::InputSelect(_) => {
                    if let Some(port) = self.in_carousel.get_item() {
                        target.input_manager.connect_input(port.clone());

                        target
                            .proxy
                            .send_event(NeothesiaEvent::MainMenu(super::Event::Play))
                            .unwrap();
                    }
                }

                Controls::Exit(_) => {
                    target.proxy.send_event(NeothesiaEvent::GoBack).unwrap();
                }
            },

            Message::BackPressed => match self.controls {
                Controls::SongSelect(_) => {
                    self.controls = Controls::Exit(ExitControls::new());
                }
                Controls::InputSelect(_) => {
                    self.controls = Controls::SongSelect(SongSelectControls::new());
                }
                Controls::Exit(_) => {
                    self.controls = Controls::SongSelect(SongSelectControls::new());
                }
            },

            Message::OutputsUpdated(outs) => {
                self.out_carousel.update(outs);
            }
        }
    }

    fn keyboard_input(&self, event: &iced_native::keyboard::Event) -> Option<Message> {
        use iced_native::keyboard::{Event, KeyCode};

        if let Event::KeyReleased { key_code, .. } = event {
            match key_code {
                KeyCode::Tab => Some(Message::FileSelectPressed),
                KeyCode::Left => Some(Message::PrevPressed),
                KeyCode::Right => Some(Message::NextPressed),
                KeyCode::Enter => Some(Message::ContinuePressed),
                KeyCode::Escape => Some(Message::BackPressed),
                _ => None,
            }
        } else {
            None
        }
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let (controls, footer) = match &mut self.controls {
            Controls::SongSelect(c) => {
                let (content, footer) =
                    c.view(&mut self.out_carousel, self.midi_file, self.play_along);
                (content, Some(footer))
            }
            Controls::InputSelect(c) => {
                let (content, footer) =
                    c.view(&mut self.in_carousel, self.midi_file, self.play_along);
                (content, Some(footer))
            }
            Controls::Exit(c) => (c.view(), None),
        };

        let main: Element<_, _> = {
            let image = image(self.logo_handle.clone());
            let image = container(image).center_x().center_y().width(Length::Fill);

            let main = column(vec![image.into(), controls])
                .width(Length::Fill)
                .spacing(40)
                .max_width(650);

            let centered_main = container(main)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y();

            centered_main.into()
        };

        let mut out = column(vec![main]);

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
        let output = text("Do you want to exit?")
            .style(Color::WHITE)
            .size(30)
            .horizontal_alignment(Horizontal::Center)
            .vertical_alignment(Vertical::Center);

        let select_row = row(vec![
            NeoBtn::new(
                &mut self.no_button,
                text("No")
                    .size(30)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .on_press(Message::BackPressed)
            .into(),
            NeoBtn::new(
                &mut self.yes_button,
                text("Yes")
                    .size(30)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center),
            )
            .width(Length::Fill)
            .on_press(Message::ContinuePressed)
            .into(),
        ])
        .spacing(5)
        .height(Length::Units(50));

        let controls = column(vec![output.into(), select_row.into()])
            .align_items(Alignment::Center)
            .width(Length::Units(500))
            .spacing(30);

        container(controls)
            .width(Length::Fill)
            .height(Length::Units(250))
            .center_x()
            .center_y()
            .into()
    }
}
