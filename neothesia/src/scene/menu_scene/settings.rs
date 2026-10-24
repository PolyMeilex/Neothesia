use std::{ops::RangeInclusive, path::PathBuf};

use crate::{
    context::Context,
    locals,
    scene::menu_scene::{MsgFn, Popup, icons, neo_btn_icon, on_async},
    utils::BoxFuture,
};
use nuon::TextAlign;
use piano_layout::Key;

use super::UiState;

#[derive(Default, Debug, Clone)]
pub enum RangeDetection {
    #[default]
    Idle,
    WaitingForKeys {
        keys: Vec<u8>,
    },
}

impl RangeDetection {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::WaitingForKeys { .. })
    }

    pub fn toggle(&mut self) {
        if self.is_active() {
            self.stop_detection();
        } else {
            self.start_detection();
        }
    }

    pub fn stop_detection(&mut self) {
        *self = RangeDetection::Idle;
    }

    pub fn start_detection(&mut self) {
        *self = RangeDetection::WaitingForKeys { keys: Vec::new() };
    }

    pub fn process_note(&mut self, note: u8) -> Option<(u8, u8)> {
        match self {
            Self::Idle => {}
            Self::WaitingForKeys { keys } => {
                keys.push(note);
                keys.sort_unstable();
                keys.dedup();

                if keys.len() == 2 {
                    let start = keys[0];
                    let end = keys[1].max(start + MIN_RANGE_LEN);

                    self.stop_detection();
                    return Some((start, end));
                }
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardSize {
    Keys88,
    Keys76,
    Keys61,
    Custom,
}

impl KeyboardSize {
    const ALL: [Self; 4] = [Self::Keys88, Self::Keys76, Self::Keys61, Self::Custom];

    fn range(self) -> Option<RangeInclusive<u8>> {
        match self {
            // A0 - C8
            Self::Keys88 => Some(21..=108),
            // E1 - G7
            Self::Keys76 => Some(28..=103),
            // C2 - C7
            Self::Keys61 => Some(36..=96),
            Self::Custom => None,
        }
    }

    pub fn from_range(range: RangeInclusive<u8>) -> Self {
        Self::ALL
            .into_iter()
            .find(|size| size.range().as_ref() == Some(&range))
            .unwrap_or(Self::Custom)
    }

    fn label(self) -> String {
        match self {
            Self::Keys88 => locals::settings::keys(88),
            Self::Keys76 => locals::settings::keys(76),
            Self::Keys61 => locals::settings::keys(61),
            Self::Custom => locals::settings::custom(),
        }
    }
}

fn segmented_control(
    ui: &mut nuon::Ui,
    id: &str,
    row_w: f32,
    row_h: f32,
    labels: &[String],
    selected: usize,
) -> Option<usize> {
    let h = 33.0;
    let padding = 3.0;
    let segment_padding = 15.0;
    let font_size = 14.6;

    let layouts: Vec<_> = labels
        .iter()
        .map(|label| {
            nuon::TextLayout::new(label.clone())
                .size(font_size)
                .text_justify(TextAlign::Center)
                .text_align(TextAlign::Center)
        })
        .collect();
    let widths: Vec<f32> = layouts
        .iter()
        .map(|layout| ui.measure_text(layout).width + segment_padding * 2.0)
        .collect();

    let w = widths.iter().sum::<f32>() + padding * 2.0;
    let x = row_w - w;
    let y = nuon::center_y(row_h, h);

    nuon::quad()
        .pos(x, y)
        .size(w, h)
        .color([12, 12, 14])
        .border_radius([8.0; 4])
        .build(ui);

    let mut clicked = None;
    let mut segment_x = x + padding;

    for (i, (layout, segment_w)) in layouts.into_iter().zip(widths).enumerate() {
        let color = if i == selected {
            [74, 68, 88, 255]
        } else {
            [0; 4]
        };

        if nuon::button()
            .id(format!("{id}-{i}"))
            .pos(segment_x, y + padding)
            .size(segment_w, h - padding * 2.0)
            .color(color)
            .hover_color(if i == selected {
                color
            } else {
                [37, 35, 42, 255]
            })
            .preseed_color(if i == selected {
                color
            } else {
                [47, 45, 54, 255]
            })
            .border_radius([6.0; 4])
            .label_layout(layout)
            .build(ui)
        {
            clicked = Some(i);
        }

        segment_x += segment_w;
    }

    clicked
}

fn button() -> nuon::Button {
    nuon::button()
        .color([74, 68, 88])
        .preseed_color([74, 68, 88])
        .hover_color([87, 81, 101])
        .border_radius([5.0; 4])
}

fn render_toggler<'a>() -> nuon::SettingsRowToggler<'a> {
    nuon::settings_row_toggler().icon_font_family("Neothesia Icons")
}

fn setting_row_button(
    ui: &mut nuon::Ui,
    row_w: f32,
    row_h: f32,
    label: impl Into<std::borrow::Cow<'static, str>>,
) -> nuon::Button {
    let h = 31.0;
    let padding = 12.0;
    let min_w = 93.0;

    let layout = nuon::TextLayout::new(label)
        .text_justify(TextAlign::Center)
        .text_align(TextAlign::Center);
    let label_w = ui.measure_text(&layout).width;
    let w = (label_w + padding * 2.0).max(min_w);

    button()
        .x(row_w - w)
        .y(nuon::center_y(row_h, h))
        .size(w, h)
        .label_layout(layout)
}

impl super::MenuScene {
    pub fn handle_range_detection_noteon(&mut self, ctx: &mut Context, note: u8) {
        if let Some((start, end)) = self.state.range_detection.process_note(note) {
            ctx.config.set_piano_range_start(start);
            ctx.config.set_piano_range_end(end);
        }
    }

    pub fn settings_page_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
        // Establish the selected output/input connection
        super::state::connect_io(&self.state, ctx);

        let win_w = ctx.window_state.logical_size.width;
        let win_h = ctx.window_state.logical_size.height;

        let bottom_bar_h = 60.0;

        nuon::translate().x(0.0).y(win_h).build(ui, |ui| {
            let padding = 10.0;
            let w = 80.0;
            let h = bottom_bar_h;

            // Bottom Margin
            nuon::translate().y(-padding).add_to_current(ui);
            nuon::translate().y(-h).add_to_current(ui);

            nuon::translate().x(0.0).build(ui, |ui| {
                nuon::translate().x(padding).add_to_current(ui);

                if neo_btn_icon(ui, w, h, icons::left_arrow_icon()) {
                    self.state.go_back();
                }

                nuon::translate().x(-w - padding).add_to_current(ui);
            });
        });

        let margin_top = 40.0;
        let body_w = 650.0;

        nuon::scroll("settings_scroll")
            .scissor_size(win_w, (win_h - bottom_bar_h).max(0.0))
            .build(ui, |ui| {
                nuon::translate()
                    .x(nuon::center_x(win_w, body_w))
                    .add_to_current(ui);
                nuon::translate().y(margin_top).add_to_current(ui);

                nuon::settings_section(locals::settings::output())
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.settings_output_section(ctx, ui, rows, spacer);
                    });

                nuon::settings_section(locals::settings::input())
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.settings_input_section(ctx, ui, rows, spacer);
                    });

                nuon::settings_section(locals::settings::note_range())
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.settings_keyboard_size_row(ctx, ui, rows);

                        spacer(ui);

                        if self.state.keyboard_size == KeyboardSize::Custom {
                            self.settings_custom_range_rows(ctx, ui, rows, spacer);
                        }

                        self.settings_calibrate_row(ctx, ui, rows);
                    });

                nuon::translate().y(10.0).add_to_current(ui);

                let keyboard_h = 100.0;
                self.keyboard_layout_preview(ctx, body_w, keyboard_h, ui);
                nuon::translate().y(keyboard_h).add_to_current(ui);

                nuon::settings_section(locals::settings::waterfall())
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        if render_toggler()
                            .icon(icons::octave_lines_icon())
                            .title(locals::settings::octave_lines())
                            .subtitle(locals::settings::octave_lines_subtitle())
                            .value(ctx.config.vertical_guidelines())
                            .build(ui, rows)
                        {
                            ctx.config
                                .set_vertical_guidelines(!ctx.config.vertical_guidelines());
                        }

                        spacer(ui);

                        if render_toggler()
                            .icon(icons::bar_lines_icon())
                            .title(locals::settings::bar_lines())
                            .subtitle(locals::settings::bar_lines_subtitle())
                            .value(ctx.config.horizontal_guidelines())
                            .build(ui, rows)
                        {
                            ctx.config
                                .set_horizontal_guidelines(!ctx.config.horizontal_guidelines());
                        }

                        spacer(ui);

                        if render_toggler()
                            .icon(icons::note_names_icon())
                            .title(locals::settings::note_names())
                            .subtitle(locals::settings::note_names_subtitle())
                            .value(ctx.config.note_labels())
                            .build(ui, rows)
                        {
                            ctx.config.set_note_labels(!ctx.config.note_labels());
                        }
                    });

                nuon::settings_section(locals::settings::keyboard())
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        if render_toggler()
                            .icon(icons::key_glow_icon())
                            .title(locals::settings::key_glow())
                            .subtitle(locals::settings::key_glow_subtitle())
                            .value(ctx.config.glow())
                            .build(ui, rows)
                        {
                            ctx.config.set_glow(!ctx.config.glow());
                        }

                        spacer(ui);

                        if render_toggler()
                            .icon(icons::chord_name_icon())
                            .title(locals::settings::chord_name())
                            .subtitle(locals::settings::chord_name_subtitle())
                            .value(ctx.config.chord_identifier())
                            .build(ui, rows)
                        {
                            ctx.config
                                .set_chord_identifier(!ctx.config.chord_identifier());
                        }
                    });
            });
    }
}

impl super::MenuScene {
    fn settings_output_picker(
        &mut self,
        ui: &mut nuon::Ui,
        ctx: &mut Context,
        row_w: f32,
        row_h: f32,
    ) {
        let btn_w = 320.0;
        let btn_h = 31.0;

        let btn_x = row_w - btn_w;
        let btn_y = nuon::center_y(row_h, btn_h);

        if button()
            .pos(btn_x, btn_y)
            .size(btn_w, btn_h)
            .id("select_output")
            .label(
                self.state
                    .selected_output
                    .as_ref()
                    .map(|o| o.to_string())
                    .unwrap_or_default(),
            )
            .text_justify(TextAlign::Start)
            .build(ui)
        {
            self.popup.toggle(Popup::OutputSelector);
        }

        nuon::label()
            .icon(icons::caret_down())
            .pos(btn_x, btn_y)
            .size(btn_w, btn_h)
            .text_justify(TextAlign::End)
            .build(ui);

        if self.popup == Popup::OutputSelector {
            nuon::layer().overlay(true).build(ui, |ui| {
                nuon::translate()
                    .x(btn_x)
                    .y(btn_y + btn_h)
                    .add_to_current(ui);

                let data = &mut self.state;

                if let Some(output) =
                    nuon::combo_list(ui, "select_output_", (btn_w, btn_h), &data.outputs)
                {
                    ctx.config
                        .set_output(output.is_not_dummy().then(|| output.to_string()));
                    data.selected_output = Some(output.clone());
                    self.popup.close();
                }
            });
        }
    }

    fn settings_output_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut nuon::Ui,
        rows: &dyn Fn(&mut nuon::Ui, nuon::SettingsRow<'_>),
        spacer: &dyn Fn(&mut nuon::Ui),
    ) {
        nuon::settings_row()
            .title(locals::settings::output())
            .body(|ui, row_w, row_h| self.settings_output_picker(ui, ctx, row_w, row_h))
            .build(ui, rows);

        let (is_synth, is_midi) = self
            .state
            .selected_output
            .as_ref()
            .map(|o| (o.is_synth(), o.is_midi()))
            .unwrap_or((false, false));

        if is_synth {
            spacer(ui);

            nuon::settings_row()
                .title(locals::settings::soundfont())
                .subtitle(
                    ctx.config
                        .soundfont_path()
                        .as_ref()
                        .and_then(|path| path.file_name())
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_default(),
                )
                .body(|ui, row_w, row_h| {
                    if setting_row_button(ui, row_w, row_h, locals::settings::select_file())
                        .build(ui)
                    {
                        self.futures
                            .push(self::open_soundfont_picker(&mut self.state));
                    }
                })
                .build(ui, rows);

            spacer(ui);

            self::update_audio_gain(
                ctx,
                nuon::settings_row_spin()
                    .title(locals::settings::audio_gain())
                    .subtitle(ctx.config.audio_gain().to_string())
                    .id("gain")
                    .build(ui, rows),
            );
        } else if is_midi {
            spacer(ui);

            if nuon::settings_row_toggler()
                .title(locals::settings::separate_channels())
                .subtitle(locals::settings::separate_channels_subtitle())
                .value(ctx.config.separate_channels())
                .build(ui, rows)
            {
                ctx.config
                    .set_separate_channels(!ctx.config.separate_channels());
            }
        }
    }
}

impl super::MenuScene {
    fn settings_input_picker(
        &mut self,
        ui: &mut nuon::Ui,
        ctx: &mut Context,
        row_w: f32,
        row_h: f32,
    ) {
        let btn_w = 320.0;
        let btn_h = 31.0;

        let btn_x = row_w - btn_w;
        let btn_y = nuon::center_y(row_h, btn_h);

        if button()
            .pos(btn_x, btn_y)
            .size(btn_w, btn_h)
            .id("select_input")
            .label(
                self.state
                    .selected_input
                    .as_ref()
                    .map(|o| o.to_string())
                    .unwrap_or_default(),
            )
            .text_justify(TextAlign::Start)
            .build(ui)
        {
            self.popup.toggle(Popup::InputSelector);
        }

        nuon::label()
            .icon(icons::caret_down())
            .pos(btn_x, btn_y)
            .size(btn_w, btn_h)
            .text_justify(TextAlign::End)
            .build(ui);

        if self.popup == Popup::InputSelector {
            nuon::layer().overlay(true).build(ui, |ui| {
                nuon::translate()
                    .x(btn_x)
                    .y(btn_y + btn_h)
                    .add_to_current(ui);

                let data = &mut self.state;

                if let Some(input) =
                    nuon::combo_list(ui, "select_input_", (btn_w, btn_h), &data.inputs)
                {
                    ctx.config.set_input(Some(&input));
                    data.selected_input = Some(input.clone());
                    self.popup.close();
                }
            });
        }
    }

    fn settings_input_section(
        &mut self,
        ctx: &mut Context,
        ui: &mut nuon::Ui,
        rows: &dyn Fn(&mut nuon::Ui, nuon::SettingsRow<'_>),
        _spacer: &dyn Fn(&mut nuon::Ui),
    ) {
        nuon::settings_row()
            .title(locals::settings::input())
            .body(|ui, row_w, row_h| self.settings_input_picker(ui, ctx, row_w, row_h))
            .build(ui, rows);
    }

    fn settings_keyboard_size_row(
        &mut self,
        ctx: &mut Context,
        ui: &mut nuon::Ui,
        rows: &dyn Fn(&mut nuon::Ui, nuon::SettingsRow<'_>),
    ) {
        nuon::settings_row()
            .title(locals::settings::keyboard_size())
            .body(|ui, row_w, row_h| {
                let labels: Vec<_> = KeyboardSize::ALL.iter().map(|size| size.label()).collect();
                let selected_idx = KeyboardSize::ALL
                    .iter()
                    .position(|size| *size == self.state.keyboard_size)
                    .unwrap_or_default();

                if let Some(i) =
                    segmented_control(ui, "keyboard-size", row_w, row_h, &labels, selected_idx)
                {
                    self.state.keyboard_size = KeyboardSize::ALL[i];

                    if let Some(range) = self.state.keyboard_size.range() {
                        ctx.config.set_piano_range_start(*range.start());
                        ctx.config.set_piano_range_end(*range.end());
                    }
                } else if self.state.keyboard_size != KeyboardSize::Custom {
                    self.state.keyboard_size = KeyboardSize::from_range(ctx.config.piano_range());
                }
            })
            .build(ui, rows);
    }

    fn settings_custom_range_rows(
        &mut self,
        ctx: &mut Context,
        ui: &mut nuon::Ui,
        rows: &dyn Fn(&mut nuon::Ui, nuon::SettingsRow<'_>),
        spacer: &dyn Fn(&mut nuon::Ui),
    ) {
        self::update_range_start(
            ctx,
            &mut self.state.range_detection,
            nuon::settings_row_spin()
                .title(locals::settings::range_start())
                .subtitle(ctx.config.piano_range().start().to_string())
                .id("range-start")
                .build(ui, rows),
        );

        spacer(ui);

        self::update_range_end(
            ctx,
            &mut self.state.range_detection,
            nuon::settings_row_spin()
                .title(locals::settings::range_end())
                .subtitle(ctx.config.piano_range().end().to_string())
                .id("range-end")
                .build(ui, rows),
        );

        spacer(ui);
    }

    fn settings_calibrate_row(
        &mut self,
        _ctx: &mut Context,
        ui: &mut nuon::Ui,
        rows: &dyn Fn(&mut nuon::Ui, nuon::SettingsRow<'_>),
    ) {
        let (title, subtitle, btn_label) = match &self.state.range_detection {
            RangeDetection::Idle => (
                locals::settings::auto_detect(),
                locals::settings::auto_detect_subtitle(),
                locals::settings::detect(),
            ),
            RangeDetection::WaitingForKeys { keys } => (
                locals::settings::detection_in_progress(keys.len(), 2),
                locals::settings::detection_subtitle(),
                locals::settings::cancel(),
            ),
        };

        nuon::settings_row()
            .title(title)
            .subtitle(subtitle)
            .body(|ui, row_w, row_h| {
                if setting_row_button(ui, row_w, row_h, btn_label)
                    .id("calibrate_btn")
                    .build(ui)
                {
                    self.state.range_detection.toggle();
                }
            })
            .build(ui, rows);
    }
}

impl super::MenuScene {
    fn keyboard_layout_preview(
        &mut self,
        ctx: &Context,
        keyboard_w: f32,
        keyboard_h: f32,
        ui: &mut nuon::Ui,
    ) {
        nuon::quad()
            .size(keyboard_w, keyboard_h)
            .color([255; 3])
            .border_radius([7.0; 4])
            .build(ui);

        let range = piano_layout::KeyboardRange::new(ctx.config.piano_range());

        let white_count = range.white_count();
        let neutral_width = keyboard_w / white_count as f32;
        let neutral_height = keyboard_h;

        let layout = piano_layout::KeyboardLayout::from_range(
            piano_layout::Sizing::new(neutral_width, neutral_height),
            range,
        );

        let mut build_key = |ui: &mut nuon::Ui, key: &Key| {
            let note = layout.range.start() + key.id() as u8;
            let x = key.x();
            let y = 0.0;
            let width = key.width();
            let height = key.height();

            let event = nuon::click_area(format!("settings-preview-key-{note}"))
                .pos(key.x(), 0.0)
                .size(key.width(), key.height())
                .build(ui);

            if event.is_press_start() {
                ctx.output_manager.connection().midi_event(
                    0.into(),
                    midi_file::midly::MidiMessage::NoteOn {
                        key: note.into(),
                        vel: 100.into(),
                    },
                );
                self.midi_input_state.note_on(note);
            }
            if event.is_press_end() {
                ctx.output_manager.connection().midi_event(
                    0.into(),
                    midi_file::midly::MidiMessage::NoteOff {
                        key: note.into(),
                        vel: 0.into(),
                    },
                );
                self.midi_input_state.note_off(note);
            }

            nuon::quad()
                .pos(x, y)
                .size(width, height)
                .color(if self.midi_input_state.is_pressed(note) {
                    [122, 104, 168, 255]
                } else if key.kind().is_sharp() {
                    [0, 0, 0, 255]
                } else {
                    [0; 4]
                })
                .build(ui);
        };

        nuon::layer().build(ui, |ui| {
            for key in layout.keys.iter().filter(|key| key.kind().is_sharp()) {
                build_key(ui, key);
            }
        });

        for key in layout.keys.iter().filter(|key| key.kind().is_neutral()) {
            build_key(ui, key);
        }

        // Key borders
        let mut neutral = layout
            .keys
            .iter()
            .filter(|key| key.kind().is_neutral())
            .peekable();

        while let Some(key) = neutral.next() {
            if neutral.peek().is_some() {
                nuon::quad()
                    .x(key.x() + key.width())
                    .y(0.0)
                    .size(1.0, key.height())
                    .color([150; 3])
                    .build(ui);
            }
        }
    }
}

pub fn update_audio_gain(ctx: &mut Context, kind: nuon::SettingsRowSpinResult) {
    match kind {
        nuon::SettingsRowSpinResult::Plus => {
            ctx.config.set_audio_gain(ctx.config.audio_gain() + 0.1);
        }
        nuon::SettingsRowSpinResult::Minus => {
            ctx.config.set_audio_gain(ctx.config.audio_gain() - 0.1);
        }
        nuon::SettingsRowSpinResult::Idle => {}
    }

    ctx.config
        .set_audio_gain((ctx.config.audio_gain() * 10.0).round() / 10.0);
}

const MIN_RANGE_LEN: u8 = 24;

pub fn update_range_start(
    ctx: &mut Context,
    range_detection: &mut RangeDetection,
    kind: nuon::SettingsRowSpinResult,
) {
    if kind != nuon::SettingsRowSpinResult::Idle {
        range_detection.stop_detection();
    }

    match kind {
        nuon::SettingsRowSpinResult::Plus => {
            let v = (ctx.config.piano_range().start() + 1).min(127);
            if v + MIN_RANGE_LEN < *ctx.config.piano_range().end() {
                ctx.config.set_piano_range_start(v);
            }
        }
        nuon::SettingsRowSpinResult::Minus => {
            ctx.config
                .set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
        }
        nuon::SettingsRowSpinResult::Idle => {}
    }
}

pub fn update_range_end(
    ctx: &mut Context,
    range_detection: &mut RangeDetection,
    kind: nuon::SettingsRowSpinResult,
) {
    if kind != nuon::SettingsRowSpinResult::Idle {
        range_detection.stop_detection();
    }

    match kind {
        nuon::SettingsRowSpinResult::Plus => {
            ctx.config
                .set_piano_range_end(ctx.config.piano_range().end() + 1);
        }
        nuon::SettingsRowSpinResult::Minus => {
            let v = ctx.config.piano_range().end().saturating_sub(1);
            if *ctx.config.piano_range().start() + MIN_RANGE_LEN < v {
                ctx.config.set_piano_range_end(v);
            }
        }
        nuon::SettingsRowSpinResult::Idle => {}
    }
}

pub fn open_soundfont_picker(data: &mut UiState) -> BoxFuture<MsgFn> {
    data.is_loading = true;
    on_async(open_sondfont_picker_fut(), |res, data, ctx| {
        if let Some(font) = res {
            ctx.config.set_soundfont_path(Some(font.clone()));
        }
        data.is_loading = false;
    })
}

async fn open_sondfont_picker_fut() -> Option<PathBuf> {
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
