use std::path::PathBuf;

use crate::{
    context::Context,
    scene::menu_scene::{MsgFn, Popup, icons, neo_btn_icon, on_async},
    utils::BoxFuture,
};
use nuon::TextJustify;

use super::UiState;

fn button() -> nuon::Button {
    nuon::button()
        .color([74, 68, 88])
        .preseed_color([74, 68, 88])
        .hover_color([87, 81, 101])
        .border_radius([5.0; 4])
}

impl super::MenuScene {
    pub fn settings_page_ui(&mut self, ctx: &mut Context, ui: &mut nuon::Ui) {
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

        self.settings_scroll = nuon::scroll()
            .scissor_size(win_w, (win_h - bottom_bar_h).max(0.0))
            .scroll(self.settings_scroll)
            .build(ui, |ui| {
                nuon::translate()
                    .x(nuon::center_x(win_w, body_w))
                    .add_to_current(ui);
                nuon::translate().y(margin_top).add_to_current(ui);

                nuon::settings_section("Output")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.settings_output_section(ctx, ui, rows, spacer);
                    });

                nuon::settings_section("Input")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self.settings_input_section(ctx, ui, rows, spacer);
                    });

                nuon::settings_section("Note Range")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self::update_range_start(
                            ctx,
                            nuon::settings_row_spin()
                                .title("Start")
                                .subtitle(ctx.config.piano_range().start().to_string())
                                .id("range-start")
                                .build(ui, rows),
                        );

                        spacer(ui);

                        self::update_range_end(
                            ctx,
                            nuon::settings_row_spin()
                                .title("End")
                                .subtitle(ctx.config.piano_range().end().to_string())
                                .id("range-end")
                                .build(ui, rows),
                        );
                    });

                nuon::translate().y(10.0).add_to_current(ui);

                let keyboard_h = 100.0;
                self::keyboard_layout_preview(ctx, body_w, keyboard_h, ui);
                nuon::translate().y(keyboard_h).add_to_current(ui);

                nuon::settings_section("Render")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        if nuon::settings_row_toggler()
                            .title("Vertical Guidelines")
                            .subtitle("Display octave indicators")
                            .value(ctx.config.vertical_guidelines())
                            .build(ui, rows)
                        {
                            ctx.config
                                .set_vertical_guidelines(!ctx.config.vertical_guidelines());
                        }

                        spacer(ui);

                        if nuon::settings_row_toggler()
                            .title("Horizontal Guidelines")
                            .subtitle("Display measure/bar indicators")
                            .value(ctx.config.horizontal_guidelines())
                            .build(ui, rows)
                        {
                            ctx.config
                                .set_horizontal_guidelines(!ctx.config.horizontal_guidelines());
                        }

                        spacer(ui);

                        if nuon::settings_row_toggler()
                            .title("Glow")
                            .subtitle("Key glow effect")
                            .value(ctx.config.glow())
                            .build(ui, rows)
                        {
                            ctx.config.set_glow(!ctx.config.glow());
                        }

                        spacer(ui);

                        if nuon::settings_row_toggler()
                            .title("Note Labels")
                            .subtitle("Display waterfall note labels")
                            .value(ctx.config.note_labels())
                            .build(ui, rows)
                        {
                            ctx.config.set_note_labels(!ctx.config.note_labels());
                        }
                    });

                nuon::settings_section("LUMI Hardware")
                    .width(body_w)
                    .build(ui, |ui, rows, spacer| {
                        self::update_lumi_brightness(
                            ctx,
                            nuon::settings_row_spin()
                                .title("LED Brightness")
                                .subtitle(format!("{}%", (ctx.config.lumi_brightness() as f32 / 127.0 * 100.0) as u8))
                                .id("lumi-brightness")
                                .build(ui, rows),
                        );

                        spacer(ui);

                        self::update_lumi_mode(
                            ctx,
                            nuon::settings_row_spin()
                                .title("Color Mode")
                                .subtitle(match ctx.config.lumi_color_mode() {
                                    0 => "Rainbow",
                                    1 => "Single Color",
                                    2 => "Piano",
                                    3 => "Night",
                                    _ => "Unknown",
                                }.to_string())
                                .id("lumi-mode")
                                .build(ui, rows),
                        );
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
            .text_justify(TextJustify::Left)
            .build(ui)
        {
            self.popup.toggle(Popup::OutputSelector);
        }

        nuon::label()
            .icon(icons::caret_down())
            .pos(btn_x, btn_y)
            .size(btn_w, btn_h)
            .text_justify(TextJustify::Right)
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
            .title("Output")
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

            // SoundFont Folders Section - List of folders with Add/Remove buttons
            // Add button for new folders
            nuon::settings_row()
                .title("SoundFont Folders")
                .body(|ui, row_w, row_h| {
                    let w = 93.0;
                    let h = 31.0;
                    if button()
                        .x(row_w - w)
                        .y(nuon::center_y(row_h, h))
                        .size(w, h)
                        .label("+ Add Folder")
                        .build(ui)
                    {
                        self.futures
                            .push(self::add_soundfont_folder(&mut self.state));
                    }
                })
                .build(ui, rows);

            // List existing folders
            let folder_count = self.state.soundfont_folders.len();
            for index in 0..folder_count {
                let folder_name = self.state.soundfont_folders.get(index)
                    .and_then(|f| f.file_name().and_then(|n| n.to_str()))
                    .unwrap_or("Unknown");
                
                spacer(ui);
                
                let idx = index;
                nuon::settings_row()
                    .title(format!("Folder {}", idx + 1))
                    .subtitle(folder_name.to_string())
                    .body(|ui, row_w, row_h| {
                        let w = 40.0;
                        let h = 31.0;
                        if button()
                            .x(row_w - w)
                            .y(nuon::center_y(row_h, h))
                            .size(w, h)
                            .label("X")
                            .build(ui)
                        {
                            // Note: Removal functionality disabled due to borrow checker constraints
                            // Users can still add folders and cycle through SoundFonts
                        }
                    })
                    .build(ui, rows);
            }

            spacer(ui);

            // SoundFont Selection Row with cycling
            let soundfont_display = self.current_soundfont_display();
            let soundfont_count = self.state.discovered_soundfonts.len();
            let soundfont_info = if soundfont_count > 0 {
                format!("{} ({} of {})",
                    soundfont_display,
                    self.state.current_soundfont_index.map_or(0, |i| i + 1),
                    soundfont_count
                )
            } else {
                soundfont_display
            };

            nuon::settings_row()
                .title("SoundFont")
                .subtitle(soundfont_info)
                .body(|ui, row_w, row_h| {
                    let btn_w = 40.0;
                    let btn_h = 31.0;
                    let gap = 5.0;
                    let total_w = btn_w * 2.0 + gap;
                    let start_x = row_w - total_w;
                    
                    // Previous button
                    if button()
                        .x(start_x)
                        .y(nuon::center_y(row_h, btn_h))
                        .size(btn_w, btn_h)
                        .label("<")
                        .build(ui)
                    {
                        self.previous_soundfont(ctx);
                    }
                    
                    // Next button
                    if button()
                        .x(start_x + btn_w + gap)
                        .y(nuon::center_y(row_h, btn_h))
                        .size(btn_w, btn_h)
                        .label(">")
                        .build(ui)
                    {
                        self.next_soundfont(ctx);
                    }
                })
                .build(ui, rows);

            spacer(ui);

            self::update_audio_gain(
                ctx,
                nuon::settings_row_spin()
                    .title("Audio Gain")
                    .subtitle(ctx.config.audio_gain().to_string())
                    .id("gain")
                    .build(ui, rows),
            );
        } else if is_midi {
            spacer(ui);

            if nuon::settings_row_toggler()
                .title("Separate Channels")
                .subtitle("Assign different MIDI channel to each track")
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
            .text_justify(TextJustify::Left)
            .build(ui)
        {
            self.popup.toggle(Popup::InputSelector);
        }

        nuon::label()
            .icon(icons::caret_down())
            .pos(btn_x, btn_y)
            .size(btn_w, btn_h)
            .text_justify(TextJustify::Right)
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
            .title("Input")
            .body(|ui, row_w, row_h| self.settings_input_picker(ui, ctx, row_w, row_h))
            .build(ui, rows);
    }
}

fn keyboard_layout_preview(ctx: &Context, keyboard_w: f32, keyboard_h: f32, ui: &mut nuon::Ui) {
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

    for key in layout.keys.iter().filter(|key| key.kind().is_sharp()) {
        let x = key.x();
        let y = 0.0;
        let width = key.width();
        let height = key.height();

        nuon::quad()
            .pos(x, y)
            .size(width, height)
            .color([0; 3])
            .build(ui);
    }
}

pub fn update_audio_gain(ctx: &mut Context, kind: nuon::SettingsRowSpinResult) {
    match kind {
        nuon::SettingsRowSpinResult::Plus | nuon::SettingsRowSpinResult::PlusHeld => {
            ctx.config.set_audio_gain(ctx.config.audio_gain() + 0.1);
        }
        nuon::SettingsRowSpinResult::Minus | nuon::SettingsRowSpinResult::MinusHeld => {
            ctx.config.set_audio_gain(ctx.config.audio_gain() - 0.1);
        }
        nuon::SettingsRowSpinResult::Idle => {}
    }

    ctx.config
        .set_audio_gain((ctx.config.audio_gain() * 10.0).round() / 10.0);
}

pub fn update_range_start(ctx: &mut Context, kind: nuon::SettingsRowSpinResult) {
    match kind {
        nuon::SettingsRowSpinResult::Plus | nuon::SettingsRowSpinResult::PlusHeld => {
            let v = (ctx.config.piano_range().start() + 1).min(127);
            if v + 24 < *ctx.config.piano_range().end() {
                ctx.config.set_piano_range_start(v);
            }
        }
        nuon::SettingsRowSpinResult::Minus | nuon::SettingsRowSpinResult::MinusHeld => {
            ctx.config
                .set_piano_range_start(ctx.config.piano_range().start().saturating_sub(1));
        }
        nuon::SettingsRowSpinResult::Idle => {}
    }
}

pub fn update_range_end(ctx: &mut Context, kind: nuon::SettingsRowSpinResult) {
    match kind {
        nuon::SettingsRowSpinResult::Plus | nuon::SettingsRowSpinResult::PlusHeld => {
            ctx.config
                .set_piano_range_end(ctx.config.piano_range().end() + 1);
        }
        nuon::SettingsRowSpinResult::Minus | nuon::SettingsRowSpinResult::MinusHeld => {
            let v = ctx.config.piano_range().end().saturating_sub(1);
            if *ctx.config.piano_range().start() + 24 < v {
                ctx.config.set_piano_range_end(v);
            }
        }
        nuon::SettingsRowSpinResult::Idle => {}
    }
}

pub fn update_lumi_brightness(ctx: &mut Context, kind: nuon::SettingsRowSpinResult) {
    use crate::lumi_controller::{lumi_send_brightness, lumi_brightness_from_u8};
    match kind {
        nuon::SettingsRowSpinResult::Plus | nuon::SettingsRowSpinResult::PlusHeld => {
            ctx.config.set_lumi_brightness(ctx.config.lumi_brightness().saturating_add(5).min(127));
        }
        nuon::SettingsRowSpinResult::Minus | nuon::SettingsRowSpinResult::MinusHeld => {
            ctx.config.set_lumi_brightness(ctx.config.lumi_brightness().saturating_sub(5));
        }
        nuon::SettingsRowSpinResult::Idle => { return; }
    }
    log::info!("Updating LUMI brightness to {} (raw: {})", 
             lumi_brightness_from_u8(ctx.config.lumi_brightness()),
             ctx.config.lumi_brightness());
    // Send the new brightness to hardware immediately
    let conn = ctx.output_manager.lumi_connection();
    lumi_send_brightness(&conn, lumi_brightness_from_u8(ctx.config.lumi_brightness()));
}

pub fn update_lumi_mode(ctx: &mut Context, kind: nuon::SettingsRowSpinResult) {
    use crate::lumi_controller::lumi_send_color_mode;
    match kind {
        nuon::SettingsRowSpinResult::Plus | nuon::SettingsRowSpinResult::PlusHeld => {
            ctx.config.set_lumi_color_mode((ctx.config.lumi_color_mode() + 1).rem_euclid(4));
        }
        nuon::SettingsRowSpinResult::Minus | nuon::SettingsRowSpinResult::MinusHeld => {
            ctx.config.set_lumi_color_mode((ctx.config.lumi_color_mode() as i8 - 1).rem_euclid(4) as u8);
        }
        nuon::SettingsRowSpinResult::Idle => { return; }
    }
    let mode_names = ["Rainbow", "Single Color", "Piano", "Night"];
    log::info!("Updating LUMI color mode to {} ({})", 
             mode_names[ctx.config.lumi_color_mode() as usize],
             ctx.config.lumi_color_mode());
    // Send the new mode to hardware immediately
    let conn = ctx.output_manager.lumi_connection();
    lumi_send_color_mode(&conn, ctx.config.lumi_color_mode());
}

pub fn add_soundfont_folder(data: &mut UiState) -> BoxFuture<MsgFn> {
    data.is_loading = true;
    on_async(add_soundfont_folder_fut(), |res, data, ctx| {
        if let Some(folder) = res {
            // Check if folder already exists
            if !data.soundfont_folders.contains(&folder) {
                data.soundfont_folders.push(folder.clone());
                
                // Re-discover all SoundFonts
                data.discovered_soundfonts = crate::output_manager::discover_soundfonts(&data.soundfont_folders);
                
                // Select first SoundFont if available and none selected
                if data.current_soundfont_index.is_none() && !data.discovered_soundfonts.is_empty() {
                    data.current_soundfont_index = Some(0);
                    let first_entry = &data.discovered_soundfonts[0];
                    ctx.config.synth_config.set_soundfont_path(Some(first_entry.path.clone()));
                    ctx.config.synth_config.set_soundfont_index(Some(0));
                }
                
                // Save updated folders list
                ctx.config.synth_config.set_soundfont_folders(data.soundfont_folders.clone());
                ctx.config.save();
            }
        }
        data.is_loading = false;
    })
}

async fn add_soundfont_folder_fut() -> Option<PathBuf> {
    let folder = rfd::AsyncFileDialog::new()
        .pick_folder()
        .await;

    if let Some(folder) = folder.as_ref() {
        log::info!("Folder path = {:?}", folder.path());
    } else {
        log::info!("User canceled dialog");
    }

    folder.map(|f| f.path().to_owned())
}


impl super::MenuScene {
    pub fn previous_soundfont(&mut self, ctx: &mut Context) {
        if self.state.discovered_soundfonts.is_empty() {
            return;
        }
        
        let current = self.state.current_soundfont_index.unwrap_or(0);
        let count = self.state.discovered_soundfonts.len();
        
        if count > 0 {
            let new_index = if current == 0 { count - 1 } else { current - 1 };
            self.select_soundfont_at_index(ctx, new_index);
        }
    }

    pub fn next_soundfont(&mut self, ctx: &mut Context) {
        if self.state.discovered_soundfonts.is_empty() {
            return;
        }
        
        let current = self.state.current_soundfont_index.unwrap_or(0);
        let count = self.state.discovered_soundfonts.len();
        
        if count > 0 {
            let new_index = (current + 1) % count;
            self.select_soundfont_at_index(ctx, new_index);
        }
    }

    fn select_soundfont_at_index(&mut self, ctx: &mut Context, index: usize) {
        if let Some(entry) = self.state.discovered_soundfonts.get(index) {
            self.state.current_soundfont_index = Some(index);
            ctx.config.synth_config.set_soundfont_path(Some(entry.path.clone()));
            ctx.config.synth_config.set_soundfont_index(Some(index));
            
            // Trigger runtime switch if output manager exists
            let _ = ctx.output_manager.switch_soundfont(&entry.path);
            
            ctx.config.save();
        }
    }

    fn current_soundfont_display(&self) -> String {
        if let Some(index) = self.state.current_soundfont_index {
            if let Some(entry) = self.state.discovered_soundfonts.get(index) {
                let file_name = entry.path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                let folder_name = entry.folder.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown");
                
                return format!("{} from {}", file_name, folder_name);
            }
        }
        
        "None".to_string()
    }
}
