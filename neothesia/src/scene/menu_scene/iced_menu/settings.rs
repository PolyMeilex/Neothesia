use std::path::PathBuf;

use iced_core::{
    alignment::{Horizontal, Vertical},
    Alignment, Length, Padding,
};
use iced_runtime::Command;
use iced_widget::{button, column as col, container, pick_list, row, toggler};
use neothesia_iced_widgets::{ActionRow, BarLayout, Element, Layout, NeoBtn, PreferencesGroup};

use crate::{context::Context, output_manager::OutputDescriptor, scene::menu_scene::icons};

use super::{centered_text, page::Page, theme, Data, InputDescriptor, Message};

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

    OpenSoundFontPicker,
    SoundFontFileLoaded(Option<PathBuf>),

    RangeStart(RangeUpdateKind),
    RangeEnd(RangeUpdateKind),
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

    fn update(data: &mut Data, msg: Event, ctx: &mut Context) -> Command<Message> {
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
                ctx.config.vertical_guidelines = v;
            }
            Event::HorizontalGuidelines(v) => {
                ctx.config.horizontal_guidelines = v;
            }
            Event::OpenSoundFontPicker => {
                data.is_loading = true;
                return open_sound_font_picker(|res| {
                    Message::SettingsPage(Event::SoundFontFileLoaded(res))
                });
            }
            Event::SoundFontFileLoaded(font) => {
                if let Some(font) = font {
                    ctx.config.soundfont_path = Some(font.clone());
                }
                data.is_loading = false;
            }
            Event::RangeStart(kind) => match kind {
                RangeUpdateKind::Add => {
                    let v = (ctx.config.piano_range().start() + 1).min(127);
                    if v + 24 < *ctx.config.piano_range().end() {
                        ctx.config.piano_range.0 = v;
                    }
                }
                RangeUpdateKind::Sub => {
                    ctx.config.piano_range.0 = ctx.config.piano_range.0.saturating_sub(1);
                }
            },
            Event::RangeEnd(kind) => match kind {
                RangeUpdateKind::Add => {
                    ctx.config.piano_range.1 = (ctx.config.piano_range.1 + 1).min(127);
                }
                RangeUpdateKind::Sub => {
                    let v = ctx.config.piano_range().end().saturating_sub(1);
                    if *ctx.config.piano_range().start() + 24 < v {
                        ctx.config.piano_range.1 = v;
                    }
                }
            },
            Event::GoBack => {
                return Command::perform(async {}, |_| Message::GoBack);
            }
        }

        Command::none()
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
            .on_press(Event::GoBack);

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
}

fn output_group<'a>(data: &'a Data, ctx: &Context) -> Element<'a, Event> {
    let output_settings = {
        let output_list = pick_list(
            data.outputs.as_ref(),
            data.selected_output.clone(),
            Event::SelectOutput,
        )
        .style(theme::pick_list());

        ActionRow::new().title("Output").suffix(output_list)
    };

    let is_synth = matches!(data.selected_output, Some(OutputDescriptor::Synth(_)));
    let synth_settings = is_synth.then(|| {
        let subtitle = ctx
            .config
            .soundfont_path
            .as_ref()
            .and_then(|path| path.file_name())
            .map(|name| name.to_string_lossy().to_string());

        let mut row = ActionRow::new().title("SoundFont").suffix(
            iced_widget::button(centered_text("Select File"))
                .style(theme::button())
                .on_press(Event::OpenSoundFontPicker),
        );

        if let Some(subtitle) = subtitle {
            row = row.subtitle(subtitle);
        }

        row
    });

    PreferencesGroup::new()
        .title("Output")
        .push(output_settings)
        .push_maybe(synth_settings)
        .build()
}

fn input_group<'a>(data: &'a Data, _ctx: &Context) -> Element<'a, Event> {
    let selected_input = data.selected_input.clone();

    let input_list = pick_list(data.inputs.as_ref(), selected_input, Event::SelectInput)
        .style(theme::pick_list());

    PreferencesGroup::new()
        .title("Input")
        .push(ActionRow::new().title("Input").suffix(input_list))
        .build()
}

fn counter<'a>(value: u8, msg: fn(RangeUpdateKind) -> Event) -> Element<'a, Event> {
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

    neothesia_iced_widgets::ScrollListener::new(row, move |delta| {
        if delta.is_sign_positive() {
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
    let vertical = toggler(
        None,
        ctx.config.vertical_guidelines,
        Event::VerticalGuidelines,
    )
    .style(theme::toggler());

    let horizontal = toggler(
        None,
        ctx.config.horizontal_guidelines,
        Event::HorizontalGuidelines,
    )
    .style(theme::toggler());

    PreferencesGroup::new()
        .title("Render")
        .push(
            ActionRow::new()
                .title("Vertical Guidelines")
                .subtitle("Display octave indicators")
                .suffix(vertical),
        )
        .push(
            ActionRow::new()
                .title("Horizontal Guidelines")
                .subtitle("Display measure/bar indicators")
                .suffix(horizontal),
        )
        .build()
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
