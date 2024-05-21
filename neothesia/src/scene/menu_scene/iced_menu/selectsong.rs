use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_widget::{button, column as col, container, row, vertical_space};

use rfd::FileDialog;

use super::{
    centered_text,
    page::{Page, PageMessage},
    theme, Data, Message,
};
use crate::{context::Context, scene::menu_scene::icons, song::Song};
use neothesia_iced_widgets::{BarLayout, Element, Layout, NeoBtn};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Event {
    GoBack,
    OpenFolderPicker,
    SetSongPath { last_opened_song: Option<PathBuf> },
    Play,
}

use crate::menu_scene::Step;

pub struct SelectsongPage;

impl Page for SelectsongPage {
    type Event = Event;

    fn update(data: &mut Data, event: Event, ctx: &mut Context) -> PageMessage {
        match event {
            Event::GoBack => PageMessage::go_to_page(Step::Main),
            Event::Play => PageMessage::go_to_page(Step::TrackSelection),
            Event::OpenFolderPicker => {
                let mut page_message = PageMessage::none();
                data.is_loading = true;
                if let Some(folder) = FileDialog::new().pick_folder() {
                    // Save the selected folder to the context or data as needed
                    ctx.config.song_directory = Some(folder);
                    page_message = PageMessage::go_to_page(Step::SelectsongPage);
                }
                data.is_loading = false;
                page_message
            }
            Event::SetSongPath { last_opened_song } => {
                // Handle the event here, update the context or data accordingly

                match last_opened_song {
                    Some(song_path) => {
                        ctx.config.last_opened_song = Some(song_path.clone());
                        match midi_file::MidiFile::new(&song_path) {
                            Ok(midi) => {
                                ctx.song = Some(Song::new(midi));
                                // Trigger navigation or any other necessary action
                                PageMessage::go_to_page(Step::SelectsongPage)
                            }
                            Err(err) => {
                                log::error!("Failed to load MIDI file: {}", err);
                                // Handle the error here
                                // For now, let's return None as a placeholder
                                PageMessage::go_to_page(Step::SelectsongPage)
                            }
                        }
                    }
                    None => {
                        ctx.config.last_opened_song = None;

                        PageMessage::go_to_page(Step::SelectsongPage)
                    }
                }
            }
        }
    }

    fn view<'a>(_data: &'a Data, ctx: &Context) -> Element<'a, Event> {
        let dir_path = match dirs::home_dir() {
            Some(mut path) => {
                if let Some(folder) = &ctx.config.song_directory {
                    path.push(folder);
                }
                path
            }
            None => {
                println!("Unable to determine home directory");
                let mut path = PathBuf::new();
                path.push("~/Music");
                path
            }
        };

        let mut song_file_name = String::new();

        if let Some(path_buf) = &ctx.config.last_opened_song {
            if let Some(file_name) = path_buf.file_name() {
                if let Some(name) = file_name.to_str() {
                    if let Some(stripped_name) = name.strip_suffix(".mid") {
                        song_file_name = stripped_name.to_string();
                    } else {
                        // If the file name doesn't end with ".mid", use the original file name
                        song_file_name = name.to_string();
                    }
                }
            }
        }

        let mut elements = Vec::new();
        if let Ok(entries) = fs::read_dir(&dir_path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if let Some(extension) = Path::new(file_name).extension() {
                            if extension == "mid" || extension == "midi" {
                                let song_name =
                                    if let Some(stripped_name) = file_name.strip_suffix(".mid") {
                                        stripped_name.to_string()
                                    } else {
                                        file_name.to_string()
                                    };
                                let button_color = if song_file_name == song_name {
                                    iced_core::Color::from_rgb8(106, 0, 163)
                                } else {
                                    iced_core::Color::from_rgb8(54, 0, 107)
                                };
                                // Create a button with the song name
                                let button = button(centered_text(&song_name))
                                    .on_press(Event::SetSongPath {
                                        last_opened_song: Some(entry.path().clone()),
                                    })
                                    .style(theme::filelist_button(button_color))
                                    .width(10000);

                                elements.push(button.into());
                            }
                        }
                    }
                }
            }
        }

        // Add the list into another scrollable for a responsive UI
        let inner_scrollable = iced_widget::Scrollable::new(
            iced_widget::Column::with_children(elements)
                .spacing(5)
                .align_items(Alignment::Start),
        )
        .height(ctx.window_state.logical_size.height as u16 - 400)
        .width(ctx.window_state.logical_size.width as u16 - 421);

        let inner_scrollable_element: Element<'_, Event> = inner_scrollable.into();

        let column = iced_widget::scrollable(iced_widget::column(vec![inner_scrollable_element]));

        let mut elements = Vec::new();

        let center_text = centered_text("Song list")
            .size(20)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let center_text_container = container(center_text)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 25.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });
        elements.push(center_text_container.into());

        let center_text = centered_text(format!("Selected song: {}", song_file_name))
            .size(12)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let center_text_container = container(center_text)
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 25.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });
        elements.push(center_text_container.into());

        elements.push(
            col![vertical_space().height(Length::Fixed(10.0)), column]
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .into(),
        );

        let mut song_directory = String::new();

        if let Some(_path_buf) = &ctx.config.song_directory {
            if let Some(path_buf) = &ctx.config.song_directory {
                song_directory = path_buf.to_string_lossy().to_string();
            }
        }
        let center_text = centered_text(format!("Midi directory path: {}", song_directory))
            .size(12)
            .width(Length::Fill)
            .horizontal_alignment(Horizontal::Center);

        let center_text_container = container(center_text)
            .width(Length::Fill)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Bottom)
            .padding(Padding {
                top: 80.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });
        elements.push(center_text_container.into());

        let column = iced_widget::scrollable(iced_widget::column(elements));

        let right = {
            let play = NeoBtn::new(
                icons::play_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::Play);

            if ctx.song.is_some() {
                row![play]
            } else {
                row![]
            }
            .spacing(10)
            .width(Length::Shrink)
            .align_items(Alignment::Center)
        };

        let right = container(right)
            .width(Length::Fill)
            .align_x(Horizontal::Right)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 0.0,
            });

        let left = {
            let back = NeoBtn::new(
                icons::left_arrow_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::GoBack);

            row![back].align_items(Alignment::Start)
        };

        let left = container(left)
            .width(Length::Fill)
            .align_x(Horizontal::Left)
            .align_y(Vertical::Center)
            .padding(Padding {
                top: 0.0,
                right: 10.0,
                bottom: 10.0,
                left: 10.0,
            });

        let center = {
            let folderbtn = NeoBtn::new(
                icons::folder_icon()
                    .size(30.0)
                    .vertical_alignment(Vertical::Center)
                    .horizontal_alignment(Horizontal::Center),
            )
            .height(Length::Fixed(60.0))
            .min_width(80.0)
            .on_press(Event::OpenFolderPicker);

            container(folderbtn)
                .width(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center)
                .padding(Padding {
                    top: 0.0,
                    right: 10.0,
                    bottom: 10.0,
                    left: 0.0,
                })
        };

        Layout::new()
            .body(column)
            .bottom(BarLayout::new().left(left).center(center).right(right))
            .into()
    }

    fn keyboard_input(event: &iced_runtime::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Enter => Some(Message::SelectsongPage(self::Event::Play)),
                Named::Escape => Some(Message::GoToPage(Step::Main)),
                _ => None,
            },
            _ => None,
        }
    }
}
