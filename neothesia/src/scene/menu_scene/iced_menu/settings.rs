use std::path::PathBuf;

use iced_core::{
    alignment::{Horizontal, Vertical},
    mouse::ScrollDelta,
    Alignment, Length, Padding,
};
use iced_runtime::Task;
use iced_widget::{button, column as col, container, mouse_area, pick_list, row, toggler};
use neothesia_iced_widgets::{ActionRow, BarLayout, Element, Layout, NeoBtn, PreferencesGroup};

use crate::{context::Context, output_manager::OutputDescriptor, scene::menu_scene::icons};

use super::{
    centered_text,
    page::{Page, PageMessage},
    theme, Data, InputDescriptor, Message,
};

#[derive(Debug, Clone)]
pub enum RangeUpdateKind {
    Add,
    Sub,
}

#[derive(Debug, Clone)]
pub enum Event {
    SelectOutput(OutputDescriptor),
    SelectInput(InputDescriptor),
    VerticalGuidelines(bool),
    HorizontalGuidelines(bool),
    SeparateChannels(bool),
    OpenSoundFontPicker,
    SoundFontFileLoaded(Option<PathBuf>),

    RangeStart(RangeUpdateKind),
    RangeEnd(RangeUpdateKind),
    AudioGain(RangeUpdateKind),
    GoBack,
}

pub struct SettingsPage;

impl SettingsPage {
    pub fn open_sound_font_picker() -> Event {
        Event::OpenSoundFontPicker
    }
}

impl Page for SettingsPage {
    type Event = Event;

    fn update(data: &mut Data, msg: Event, ctx: &mut Context) -> PageMessage {
        match msg {
            Event::SelectOutput(output) => {
                ctx.config
                    .set_output(if let OutputDescriptor::DummyOutput = output {
                        None
                    } else {
                        Some(output.to_string())
                    });
                data.selected_output = Some(output);
            }
            Event::SelectInput(input) => {
                ctx.config.set_input(Some(&input));
                data.selected_input = Some(input);
            }
            Event::VerticalGuidelines(v) => {
                ctx.config.set_vertical_guidelines(v);
            }
            Event::HorizontalGuidelines(v) => {
                ctx.config.set_horizontal_guidelines(v);
            }
            Event::SeparateChannels(v) => {
                ctx.config.set_separate_channels(v);
            }
            Event::OpenSoundFontPicker => {
                data.is_loading = true;

                let cmd = Task::perform(open_sound_font_picker(), Event::SoundFontFileLoaded)
                    .map(Message::SettingsPage);
                return PageMessage::Command(cmd);
            }
            Event::SoundFontFileLoaded(font) => {
                if let Some(font) = font {
                    ctx.config.set_soundfont_path(Some(font.clone()));
                }
                data.is_loading = false;
            }
            Event::RangeStart(kind) => match kind {
                RangeUpdateKind::Add => {
                    let v = (ctx.config.piano_range().start() + 1).min(127);
                    if v + 24 < *ctx.config.piano_range().end() {
                        ctx.config.set_piano_range_start(v);
                    }
                }
                RangeUpdateKind::Sub => {
                    ctx.config
                        .set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
                }
            },
            Event::RangeEnd(kind) => match kind {
                RangeUpdateKind::Add => {
                    ctx.config
                        .set_piano_range_end(ctx.config.piano_range().end() + 1);
                }
                RangeUpdateKind::Sub => {
                    let v = ctx.config.piano_range().end().saturating_sub(1);
                    if *ctx.config.piano_range().start() + 24 < v {
                        ctx.config.set_piano_range_end(v);
                    }
                }
            },
            Event::AudioGain(kind) => {
                match kind {
                    RangeUpdateKind::Add => {
                        ctx.config.set_audio_gain(ctx.config.audio_gain() + 0.1);
                    }
                    RangeUpdateKind::Sub => {
                        ctx.config.set_audio_gain(ctx.config.audio_gain() - 0.1);
                    }
                }

                ctx.config
                    .set_audio_gain((ctx.config.audio_gain() * 10.0).round() / 10.0);
            }
            Event::GoBack => {
                return PageMessage::go_back();
            }
        }

        PageMessage::none()
    }

    fn view<'a>(data: &'a Data, ctx: &Context) -> Element<'a, Event> {
        let output_group = output_group(data, ctx);
        let input_group = input_group(data, ctx);
        let note_range_group = note_range_group(data, ctx);
        let guidelines_group = guidelines_group(data, ctx);
        let range = neothesia_iced_widgets::PianoRange(ctx.config.piano_range());

        let column = col![
            output_group,
            input_group,
            note_range_group,
            range,
            guidelines_group,
        ]
        .spacing(10)
        .width(Length::Fill)
        .align_x(Alignment::Center);

        let left = {
            let back = NeoBtn::new(icons::left_arrow_icon().size(30.0).center())
                .height(Length::Fixed(60.0))
                .min_width(80.0)
                .on_press(Event::GoBack);

            row![back]
                .spacing(10)
                .width(Length::Shrink)
                .align_y(Alignment::Center)
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

        let body = col![body].width(Length::Fill).align_x(Alignment::Center);

        let column = iced_widget::scrollable(body).style(theme::scrollable);

        Layout::new()
            .body(column)
            .bottom(BarLayout::new().left(left))
            .into()
    }

    fn keyboard_input(event: &iced_runtime::keyboard::Event, _ctx: &Context) -> Option<Message> {
        use iced_runtime::keyboard::{key::Named, Event, Key};

        match event {
            Event::KeyPressed {
                key: Key::Named(key),
                ..
            } => match key {
                Named::Tab => Some(Message::SettingsPage(SettingsPage::open_sound_font_picker())),
                Named::Escape => Some(Message::GoBack),
                _ => None,
            },
            _ => None,
        }
    }
}

fn output_group<'a>(data: &'a Data, ctx: &Context) -> Element<'a, Event> {
    let output_settings = {
        let output_list = pick_list(
            data.outputs.as_ref(),
            data.selected_output.clone(),
            Event::SelectOutput,
        )
        .style(theme::pick_list)
        .menu_style(theme::pick_list_menu);

        ActionRow::new().title("Output").suffix(output_list)
    };

    let is_synth = matches!(data.selected_output, Some(OutputDescriptor::Synth(_)));
    let synth_settings = is_synth.then(|| {
        let subtitle = ctx
            .config
            .soundfont_path()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string());

        let mut row = ActionRow::new().title("SoundFont").suffix(
            iced_widget::button(centered_text("Select File"))
                .style(theme::button)
                .on_press(Event::OpenSoundFontPicker),
        );

        if let Some(subtitle) = subtitle {
            row = row.subtitle(subtitle);
        }

        row
    });
    let synth_gain_settings = is_synth.then(|| {
        ActionRow::new()
            .title("Audio Gain")
            .suffix(counter(ctx.config.audio_gain(), Event::AudioGain))
    });

    let is_midi = matches!(data.selected_output, Some(OutputDescriptor::MidiOut(_)));
    let separate_channels_toggler = toggler(ctx.config.separate_channels())
        .on_toggle(Event::SeparateChannels)
        .style(theme::toggler);

    let separate_channels_settings = mouse_area(
        ActionRow::new()
            .title("Separate Channels")
            .subtitle("Assign different MIDI channel to each track")
            .suffix(separate_channels_toggler),
    )
    .on_press(Event::SeparateChannels(!ctx.config.separate_channels()));
    let separate_channels_settings = is_midi.then_some(separate_channels_settings);

    PreferencesGroup::new()
        .title("Output")
        .push(output_settings)
        .push_maybe(synth_settings)
        .push_maybe(synth_gain_settings)
        .push_maybe(separate_channels_settings)
        .build()
}

fn input_group<'a>(data: &'a Data, _ctx: &Context) -> Element<'a, Event> {
    let selected_input = data.selected_input.clone();

    let input_list = pick_list(data.inputs.as_ref(), selected_input, Event::SelectInput)
        .style(theme::pick_list)
        .menu_style(theme::pick_list_menu);

    PreferencesGroup::new()
        .title("Input")
        .push(ActionRow::new().title("Input").suffix(input_list))
        .build()
}

fn counter<'a>(value: impl ToString, msg: fn(RangeUpdateKind) -> Event) -> Element<'a, Event> {
    let label = centered_text(value);
    let sub = button(centered_text("-").width(30).height(30))
        .padding(0)
        .style(theme::round_button)
        .on_press(msg(RangeUpdateKind::Sub));
    let add = button(centered_text("+").width(30).height(30))
        .padding(0)
        .style(theme::round_button)
        .on_press(msg(RangeUpdateKind::Add));

    let row = row![label, sub, add].spacing(10).align_y(Alignment::Center);

    mouse_area(row)
        .on_scroll(move |delta| {
            let (ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. }) = delta;

            if y.is_sign_positive() {
                msg(RangeUpdateKind::Add)
            } else {
                msg(RangeUpdateKind::Sub)
            }
        })
        .into()
}

fn note_range_group<'a>(_data: &'a Data, ctx: &Context) -> Element<'a, Event> {
    let start = counter(*ctx.config.piano_range().start(), Event::RangeStart);
    let end = counter(*ctx.config.piano_range().end(), Event::RangeEnd);

    PreferencesGroup::new()
        .title("Note Range")
        .push(ActionRow::new().title("Start").suffix(start))
        .push(ActionRow::new().title("End").suffix(end))
        .build()
}

fn guidelines_group<'a>(_data: &'a Data, ctx: &Context) -> Element<'a, Event> {
    let vertical = toggler(ctx.config.vertical_guidelines())
        .on_toggle(Event::VerticalGuidelines)
        .style(theme::toggler);

    let horizontal = toggler(ctx.config.horizontal_guidelines())
        .on_toggle(Event::HorizontalGuidelines)
        .style(theme::toggler);

    PreferencesGroup::new()
        .title("Render")
        .push(
            mouse_area(
                ActionRow::new()
                    .title("Vertical Guidelines")
                    .subtitle("Display octave indicators")
                    .suffix(vertical),
            )
            .on_press(Event::VerticalGuidelines(!ctx.config.vertical_guidelines())),
        )
        .push(
            mouse_area(
                ActionRow::new()
                    .title("Horizontal Guidelines")
                    .subtitle("Display measure/bar indicators")
                    .suffix(horizontal),
            )
            .on_press(Event::HorizontalGuidelines(
                !ctx.config.horizontal_guidelines(),
            )),
        )
        .build()
}

async fn open_sound_font_picker() -> Option<PathBuf> {
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
}
