use std::path::PathBuf;

use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_runtime::Command;
use iced_widget::{column as col, container, pick_list, row, toggler};

use crate::{
    iced_utils::iced_state::Element,
    output_manager::OutputDescriptor,
    scene::menu_scene::{
        icons,
        layout::{BarLayout, Layout},
        neo_btn::NeoBtn,
        preferences_group,
    },
    target::Target,
};

use super::{centered_text, theme, top_padded, Data, InputDescriptor, Message};

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    SelectOutput(OutputDescriptor),
    SelectInput(InputDescriptor),
    VerticalGuidelines(bool),

    OpenSoundFontPicker,
    SoundFontFileLoaded(Option<PathBuf>),
}

impl From<SettingsMessage> for Message {
    fn from(msg: SettingsMessage) -> Self {
        Message::Settings(msg)
    }
}

pub(super) fn update(
    data: &mut Data,
    msg: SettingsMessage,
    target: &mut Target,
) -> Command<Message> {
    match msg {
        SettingsMessage::SelectOutput(output) => {
            target
                .config
                .set_output(if let OutputDescriptor::DummyOutput = output {
                    None
                } else {
                    Some(output.to_string())
                });
            data.selected_output = Some(output);
        }
        SettingsMessage::SelectInput(input) => {
            target.config.set_input(Some(&input));
            data.selected_input = Some(input);
        }
        SettingsMessage::VerticalGuidelines(v) => {
            target.config.vertical_guidelines = v;
        }
        SettingsMessage::OpenSoundFontPicker => {
            data.is_loading = true;
            return open_sound_font_picker(|res| {
                Message::Settings(SettingsMessage::SoundFontFileLoaded(res))
            });
        }
        SettingsMessage::SoundFontFileLoaded(font) => {
            if let Some(font) = font {
                target.config.soundfont_path = Some(font.clone());
            }
            data.is_loading = false;
        }
    }

    Command::none()
}

pub(super) fn view<'a>(data: &'a Data, target: &Target) -> Element<'a, Message> {
    let output_list = {
        let outputs = &data.outputs;
        let selected_output = data.selected_output.clone();

        let is_synth = matches!(selected_output, Some(OutputDescriptor::Synth(_)));

        let output_list = pick_list(outputs, selected_output, |v| {
            SettingsMessage::SelectOutput(v).into()
        })
        .style(theme::pick_list());

        let mut group = preferences_group::PreferencesGroup::new()
            .title("Output")
            .push(
                preferences_group::ActionRow::new()
                    .title("Output")
                    .suffix(output_list),
            );

        if is_synth {
            let subtitle = target
                .config
                .soundfont_path
                .as_ref()
                .and_then(|path| path.file_name())
                .map(|name| name.to_string_lossy().to_string());

            let mut row = preferences_group::ActionRow::new()
                .title("SourdFont")
                .suffix(
                    iced_widget::button(centered_text("Select File"))
                        .style(theme::button())
                        .on_press(SettingsMessage::OpenSoundFontPicker.into()),
                );

            if let Some(subtitle) = subtitle {
                row = row.subtitle(subtitle);
            }

            group = group.push(row);
        }

        group.build()
    };

    let input_list = {
        let inputs = &data.inputs;
        let selected_input = data.selected_input.clone();

        let input_list = pick_list(inputs, selected_input, |v| {
            SettingsMessage::SelectInput(v).into()
        })
        .style(theme::pick_list());

        preferences_group::PreferencesGroup::new()
            .title("Input")
            .push(
                preferences_group::ActionRow::new()
                    .title("Input")
                    .suffix(input_list),
            )
            .build()
    };

    let guidelines = {
        let toggler = toggler(None, target.config.vertical_guidelines, |v| {
            SettingsMessage::VerticalGuidelines(v).into()
        })
        .style(theme::toggler());

        preferences_group::PreferencesGroup::new()
            .title("Render")
            .push(
                preferences_group::ActionRow::new()
                    .title("Vertical Guidelines")
                    .subtitle("Display octave indicators")
                    .suffix(toggler),
            )
            .build()
    };

    let column = col![output_list, input_list, guidelines]
        .spacing(10)
        .width(Length::Fill)
        .align_items(Alignment::Center);

    let left = {
        let back = NeoBtn::new(
            icons::left_arrow_icon()
                .size(30.0)
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center),
        )
        .height(Length::Fixed(60.0))
        .min_width(80.0)
        .on_press(Message::GoToPage(super::Step::Main));

        row![back]
            .spacing(10)
            .width(Length::Shrink)
            .align_items(Alignment::Center)
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

    Layout::new()
        .body(top_padded(column))
        .bottom(BarLayout::new().left(left))
        .into()
}

fn open_sound_font_picker(
    f: impl FnOnce(Option<PathBuf>) -> Message + 'static + Send,
) -> Command<Message> {
    Command::perform(
        async {
            let file = rfd::AsyncFileDialog::new()
                .add_filter("SoundFont2", &["sf2"])
                .pick_file()
                .await;

            if let Some(file) = file.as_ref() {
                log::info!("Font path = {:?}", file.path());
            } else {
                log::info!("User canceled dialog");
            }

            file.map(|f| f.path().to_owned())
        },
        f,
    )
}
