use std::path::PathBuf;

use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_runtime::Command;
use iced_widget::{button, column as col, container, pick_list, row, toggler};

use crate::{
    iced_utils::iced_state::Element,
    output_manager::OutputDescriptor,
    scene::menu_scene::{
        icons,
        layout::{BarLayout, Layout, PushIf},
        neo_btn::NeoBtn,
        preferences_group, scroll_listener,
    },
    target::Target,
};

use super::{centered_text, theme, Data, InputDescriptor, Message};

#[derive(Debug, Clone)]
pub enum RangeUpdateKind {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    SelectOutput(OutputDescriptor),
    SelectInput(InputDescriptor),
    VerticalGuidelines(bool),
    HorizontalGuidelines(bool),

    OpenSoundFontPicker,
    SoundFontFileLoaded(Option<PathBuf>),

    RangeStart(RangeUpdateKind),
    RangeEnd(RangeUpdateKind),
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
        SettingsMessage::HorizontalGuidelines(v) => {
            target.config.horizontal_guidelines = v;
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
        SettingsMessage::RangeStart(kind) => match kind {
            RangeUpdateKind::Add => {
                let v = (target.config.piano_range().start() + 1).min(127);
                if v + 24 < *target.config.piano_range().end() {
                    target.config.piano_range.0 = v;
                }
            }
            RangeUpdateKind::Sub => {
                target.config.piano_range.0 = target.config.piano_range.0.saturating_sub(1);
            }
        },
        SettingsMessage::RangeEnd(kind) => match kind {
            RangeUpdateKind::Add => {
                target.config.piano_range.1 = (target.config.piano_range.1 + 1).min(127);
            }
            RangeUpdateKind::Sub => {
                let v = target.config.piano_range().end().saturating_sub(1);
                if *target.config.piano_range().start() + 24 < v {
                    target.config.piano_range.1 = v;
                }
            }
        },
    }

    Command::none()
}

fn output_group<'a>(data: &'a Data, target: &Target) -> Element<'a, SettingsMessage> {
    let output_settings = {
        let output_list = pick_list(
            &data.outputs,
            data.selected_output.clone(),
            SettingsMessage::SelectOutput,
        )
        .style(theme::pick_list());

        preferences_group::ActionRow::new()
            .title("Output")
            .suffix(output_list)
    };

    let is_synth = matches!(data.selected_output, Some(OutputDescriptor::Synth(_)));
    let synth_settings = is_synth.then(|| {
        let subtitle = target
            .config
            .soundfont_path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string());

        let mut row = preferences_group::ActionRow::new()
            .title("SoundFont")
            .suffix(
                iced_widget::button(centered_text("Select File"))
                    .style(theme::button())
                    .on_press(SettingsMessage::OpenSoundFontPicker),
            );

        if let Some(subtitle) = subtitle {
            row = row.subtitle(subtitle);
        }

        row
    });

    preferences_group::PreferencesGroup::new()
        .title("Output")
        .push(output_settings)
        .push_if(synth_settings)
        .build()
}

fn input_group<'a>(data: &'a Data, _target: &Target) -> Element<'a, SettingsMessage> {
    let inputs = &data.inputs;
    let selected_input = data.selected_input.clone();

    let input_list =
        pick_list(inputs, selected_input, SettingsMessage::SelectInput).style(theme::pick_list());

    preferences_group::PreferencesGroup::new()
        .title("Input")
        .push(
            preferences_group::ActionRow::new()
                .title("Input")
                .suffix(input_list),
        )
        .build()
}

fn counter<'a>(
    value: u8,
    msg: fn(RangeUpdateKind) -> SettingsMessage,
) -> Element<'a, SettingsMessage> {
    let label = centered_text(value);
    let sub = button(centered_text("-").width(30).height(30))
        .padding(0)
        .style(theme::round_button())
        .on_press(msg(RangeUpdateKind::Sub));
    let add = button(centered_text("+").width(30).height(30))
        .padding(0)
        .style(theme::round_button())
        .on_press(msg(RangeUpdateKind::Add));

    let row = row![label, sub, add]
        .spacing(10)
        .align_items(Alignment::Center);

    scroll_listener::ScrollListener::new(row, move |delta| {
        if delta.is_sign_positive() {
            msg(RangeUpdateKind::Add)
        } else {
            msg(RangeUpdateKind::Sub)
        }
    })
    .into()
}

fn note_range_group<'a>(_data: &'a Data, target: &Target) -> Element<'a, SettingsMessage> {
    let start = counter(
        *target.config.piano_range().start(),
        SettingsMessage::RangeStart,
    );
    let end = counter(
        *target.config.piano_range().end(),
        SettingsMessage::RangeEnd,
    );

    preferences_group::PreferencesGroup::new()
        .title("Note Range")
        .push(
            preferences_group::ActionRow::new()
                .title("Start")
                .suffix(start),
        )
        .push(preferences_group::ActionRow::new().title("End").suffix(end))
        .build()
}

fn guidelines_group<'a>(_data: &'a Data, target: &Target) -> Element<'a, SettingsMessage> {
    let vertical = toggler(
        None,
        target.config.vertical_guidelines,
        SettingsMessage::VerticalGuidelines,
    )
    .style(theme::toggler());

    let horizontal = toggler(
        None,
        target.config.horizontal_guidelines,
        SettingsMessage::HorizontalGuidelines,
    )
    .style(theme::toggler());

    preferences_group::PreferencesGroup::new()
        .title("Render")
        .push(
            preferences_group::ActionRow::new()
                .title("Vertical Guidelines")
                .subtitle("Display octave indicators")
                .suffix(vertical),
        )
        .push(
            preferences_group::ActionRow::new()
                .title("Horizontal Guidelines")
                .subtitle("Display measure/bar indicators")
                .suffix(horizontal),
        )
        .build()
}

pub(super) fn view<'a>(data: &'a Data, target: &Target) -> Element<'a, Message> {
    let output_group = output_group(data, target);
    let input_group = input_group(data, target);
    let note_range_group = note_range_group(data, target);
    let guidelines_group = guidelines_group(data, target);
    let range = super::super::piano_range::PianoRange(target.config.piano_range());

    let column = col![
        output_group,
        input_group,
        note_range_group,
        range,
        guidelines_group,
    ]
    .spacing(10)
    .width(Length::Fill)
    .align_items(Alignment::Center);

    let column: Element<SettingsMessage> = column.into();
    let column: Element<Message> = column.map(Message::Settings);

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

    let body = container(column).max_width(650).padding(Padding {
        top: 50.0,
        ..Padding::ZERO
    });

    let body = col![body]
        .width(Length::Fill)
        .align_items(Alignment::Center);

    let column = iced_widget::scrollable(body);

    Layout::new()
        .body(column)
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
