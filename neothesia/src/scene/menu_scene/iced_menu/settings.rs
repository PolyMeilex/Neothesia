use std::path::PathBuf;

use iced_core::{alignment::Vertical, text::LineHeight, Alignment, Length};
use iced_runtime::Command;
use iced_widget::{button, column as col, image, pick_list, row, text, toggler};

use crate::{
    iced_utils::iced_state::Element, output_manager::OutputDescriptor,
    scene::menu_scene::neo_btn::neo_button, target::Target,
};

use super::{center_x, centered_text, theme, top_padded, Data, InputDescriptor, Message, Step};

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
        .width(Length::Fill)
        .style(theme::pick_list());

        let output_title = text("Output:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Fixed(30.0));

        if is_synth {
            let btn = button(centered_text("SoundFont"))
                .width(Length::Fixed(50.0))
                .on_press(SettingsMessage::OpenSoundFontPicker.into())
                .style(theme::button());

            row![
                output_title.width(Length::Fixed(60.0)),
                output_list.width(Length::FillPortion(3)),
                btn.width(Length::FillPortion(1))
            ]
        } else {
            row![output_title, output_list]
        }
        .spacing(10)
    };

    let input_list = {
        let inputs = &data.inputs;
        let selected_input = data.selected_input.clone();

        let input_list = pick_list(inputs, selected_input, |v| {
            SettingsMessage::SelectInput(v).into()
        })
        .width(Length::Fill)
        .style(theme::pick_list());

        let input_title = text("Input:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Fixed(30.0));

        row![
            input_title.width(Length::Fixed(60.0)),
            input_list.width(Length::FillPortion(3)),
        ]
        .spacing(10)
    };

    let guidelines = {
        let title = text("Guidelines:")
            .vertical_alignment(Vertical::Center)
            .height(Length::Fixed(30.0));

        let toggler = toggler(
            Some("Vertical".to_string()),
            target.config.vertical_guidelines,
            |v| SettingsMessage::VerticalGuidelines(v).into(),
        )
        .text_line_height(LineHeight::Absolute(30.0.into()));

        row![title, toggler].spacing(10)
    };

    let buttons = row![neo_button("Back")
        .on_press(Message::GoToPage(Step::Main))
        .width(Length::Fill),]
    .width(Length::Shrink)
    .height(Length::Fixed(50.0));

    let column = col![
        image(data.logo_handle.clone()),
        col![output_list, input_list, guidelines].spacing(10),
        buttons,
    ]
    .spacing(40)
    .align_items(Alignment::Center);

    center_x(top_padded(column)).into()
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
