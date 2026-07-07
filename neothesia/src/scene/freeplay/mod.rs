use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use lilt::{Animated, Easing};
use midi_file::midly::MidiMessage;
use neothesia_core::render::{
    GlowRenderer, GuidelineRenderer, NoteLabels, QuadRenderer, TextRenderer, WaterfallRenderer,
};
use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use crate::{
    NeothesiaEvent,
    context::Context,
    icons,
    scene::{
        MouseToMidiEventState, NuonRenderer, Scene,
        freeplay::recorder::{FreeplayRecorder, PreviewState, PreviewUiState},
        playing_scene::{Keyboard, midi_player::MidiPlayer},
    },
    song::Song,
    utils::window::WinitEvent,
};

mod recorder;

pub struct FreeplayScene {
    keyboard: Keyboard,
    guidelines: GuidelineRenderer,

    text_renderer: TextRenderer,
    quad_renderer_bg: QuadRenderer,
    quad_renderer_fg: QuadRenderer,
    glow: Option<GlowRenderer>,

    // TODO: This does not make sens, but get's us going without refactoring
    song: Option<Song>,

    nuon_renderer: NuonRenderer,
    nuon: nuon::Ui,
    mouse_to_midi_state: MouseToMidiEventState,
    deduced_chord_name: String,
    recorder: FreeplayRecorder,
    recorder_status: String,
    preview_state: Option<PreviewState>,
    preview_bar_expand_animation: Animated<bool, Instant>,
}

impl FreeplayScene {
    fn preview_bar_button() -> nuon::Button {
        nuon::button().size(30.0, 30.0).border_radius([5.0; 4])
    }

    pub fn new(ctx: &mut Context, song: Option<Song>) -> Self {
        let mut keyboard = Keyboard::new(ctx, Default::default());
        keyboard.set_pressed_by_user_colors(ctx.config.color_schema()[0].clone());

        let keyboard_layout = keyboard.layout();

        let guidelines = GuidelineRenderer::new(
            keyboard_layout.clone(),
            *keyboard.pos(),
            ctx.config.vertical_guidelines(),
            false,
            Default::default(),
        );

        let text_renderer = ctx.text_renderer_factory.new_renderer();

        let quad_renderer_bg = ctx.quad_renderer_facotry.new_renderer();
        let quad_renderer_fg = ctx.quad_renderer_facotry.new_renderer();

        let glow = ctx.config.glow().then_some(GlowRenderer::new(
            &ctx.gpu,
            &ctx.transform,
            keyboard.layout(),
        ));

        Self {
            keyboard,
            guidelines,
            text_renderer,
            quad_renderer_bg,
            quad_renderer_fg,
            glow,
            song,
            nuon_renderer: NuonRenderer::new(ctx),
            nuon: nuon::Ui::new(),
            mouse_to_midi_state: MouseToMidiEventState::default(),
            deduced_chord_name: String::new(),
            recorder: FreeplayRecorder::default(),
            recorder_status: "Press REC to start recording".to_string(),
            preview_state: None,
            preview_bar_expand_animation: Animated::new(false)
                .duration(1000.)
                .easing(Easing::EaseOutExpo)
                .delay(30.0),
        }
    }

    fn clear_preview(&mut self) {
        self.preview_state = None;
        self.keyboard.set_song_config(Default::default());
        self.keyboard.reset_notes();
    }

    fn rebuild_preview(&mut self, ctx: &Context) -> Result<(), String> {
        self.clear_preview();

        let song = self.recorder.to_song()?;
        self.keyboard.set_song_config(song.config.clone());

        let hidden_tracks: Vec<usize> = song
            .config
            .tracks
            .iter()
            .filter(|track| !track.visible)
            .map(|track| track.track_id)
            .collect();

        let mut waterfall = WaterfallRenderer::new(
            &ctx.gpu,
            &song.file.tracks,
            &hidden_tracks,
            &ctx.config,
            &ctx.transform,
            self.keyboard.layout().clone(),
        );

        let note_labels = ctx.config.note_labels().then_some(NoteLabels::new(
            *self.keyboard.pos(),
            waterfall.notes(),
            ctx.text_renderer_factory.new_renderer(),
        ));

        let mut player = MidiPlayer::new_with_lead_in(
            ctx.output_manager.connection().clone(),
            song,
            self.keyboard.layout().range.clone(),
            ctx.config.separate_channels(),
            Duration::ZERO,
        );
        player.pause();
        waterfall.update(player.time_without_lead_in() + ctx.config.animation_offset());

        self.preview_state = Some(PreviewState {
            player,
            waterfall,
            note_labels,
        });
        Ok(())
    }

    fn toggle_preview_playback(&mut self) {
        let Some(player) = self.preview_state.as_mut().map(|state| &mut state.player) else {
            return;
        };

        if player.is_finished() {
            player.set_time(Duration::ZERO);
            self.keyboard.reset_notes();
        }

        player.pause_resume();
    }

    fn seek_preview_to_cursor(&mut self, ctx: &Context) {
        let Some(player) = self.preview_state.as_mut().map(|state| &mut state.player) else {
            return;
        };

        let width = ctx.window_state.logical_size.width.max(1.0);
        let percentage = (ctx.window_state.cursor_logical_position.x / width).clamp(0.0, 1.0);

        player.set_percentage_time(percentage);
        self.keyboard.reset_notes();
    }

    fn preview_ui_state(&self) -> PreviewUiState {
        if let Some(player) = self.preview_state.as_ref().map(|state| &state.player) {
            return PreviewUiState {
                available: true,
                is_paused: player.is_paused(),
                progress: player.percentage().clamp(0.0, 1.0),
                current_time: player.time().as_secs_f32(),
                length: player.length().as_secs_f32(),
                song_name: player.song().file.name.clone(),
                measures: player.song().file.measures.clone(),
            };
        }

        let song_name = if self.recorder.is_recording() {
            "Preview unavailable while recording".to_string()
        } else {
            "Stop recording to enable preview".to_string()
        };

        PreviewUiState {
            available: false,
            is_paused: true,
            progress: 0.0,
            current_time: 0.0,
            length: self.recorder.duration().as_secs_f32(),
            song_name,
            measures: Arc::from([]),
        }
    }

    fn preview_status_label(&self) -> String {
        if self.recorder.is_recording() {
            return format!(
                "Recording {:.1}s | {} events",
                self.recorder.duration().as_secs_f32(),
                self.recorder.event_count()
            );
        }

        self.recorder_status.clone()
    }

    fn record_button_label(&self) -> &'static str {
        if self.recorder.is_recording() {
            "STOP"
        } else {
            "REC"
        }
    }

    fn stop_recording(&mut self, ctx: &Context) -> Result<(), String> {
        self.recorder.stop();
        self.rebuild_preview(ctx)
    }

    fn handle_record_click(&mut self, ctx: &Context) {
        if self.recorder.is_recording() {
            match self.stop_recording(ctx) {
                Ok(()) => {
                    self.recorder_status = format!(
                        "Recorded {:.1}s. Press PLAY to listen back or REC to record again. Recording again discards the current take, so SAVE it first if you need it.",
                        self.recorder.duration().as_secs_f32()
                    );
                }
                Err(err) => {
                    self.recorder_status = err;
                }
            }

            return;
        }

        self.clear_preview();
        self.recorder.start();
        self.recorder_status = "Recording in progress".to_string();
    }

    fn save_dialog(&self, ctx: &Context) -> rfd::FileDialog {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("midi", &["mid", "midi"])
            .set_file_name("freeplay-recording.mid");

        if let Some(path) = ctx.config.last_opened_song().and_then(|path| path.parent()) {
            dialog = dialog.set_directory(path);
        }

        dialog
    }

    fn handle_save_click(&mut self, ctx: &Context) {
        if self.recorder.is_recording()
            && let Err(err) = self.stop_recording(ctx)
        {
            self.recorder_status = err;
            return;
        }

        if !self.recorder.has_note_events() {
            self.recorder_status = "Nothing recorded yet".to_string();
            return;
        }

        // TODO: This block the whole app, causing the app to become unresponsive during file pick. Switch to proper async rfd dialog
        match self.save_dialog(ctx).save_file() {
            Some(path) => match self.recorder.save_to_path(&path) {
                Ok(()) => {
                    self.recorder_status = format!("Saved recording to {}", path.display());
                }
                Err(err) => {
                    self.recorder_status = err;
                }
            },
            None => {
                self.recorder_status = "Save canceled".to_string();
            }
        }
    }

    fn update_preview(&mut self, ctx: &Context, delta: Duration) -> Option<f32> {
        let player = &mut self.preview_state.as_mut()?.player;

        let midi_events = player.update(delta);
        self.keyboard.file_midi_events(&ctx.config, &midi_events);

        if player.is_finished() && !player.is_paused() {
            player.pause();
        }

        Some(player.time_without_lead_in() + ctx.config.animation_offset())
    }

    fn update_preview_ui(&mut self, ctx: &mut Context) {
        let top_bar_height = 75.0;
        let is_hovered = ctx.window_state.cursor_logical_position.y < top_bar_height * 1.7;
        self.preview_bar_expand_animation
            .transition(is_hovered, ctx.frame_timestamp);

        let width = ctx.window_state.logical_size.width;
        let preview = self.preview_ui_state();

        // TODO: Replace separate bools with singular enum Msg
        let mut toggle_play = false;
        let mut seek = false;
        let mut go_back = false;
        let mut record_clicked = false;
        let mut save_clicked = false;
        let record_label = self.record_button_label();
        let status_label = self.preview_status_label();

        nuon::translate()
            .y(self
                .preview_bar_expand_animation
                .animate_bool(-70.0, 0.0, ctx.frame_timestamp))
            .build(&mut self.nuon, |ui| {
                nuon::quad()
                    .size(width, top_bar_height)
                    .color([37, 35, 42])
                    .build(ui);

                if Self::preview_bar_button()
                    .icon(icons::left_arrow_icon())
                    .build(ui)
                {
                    go_back = true;
                }

                nuon::label()
                    .x(40.0)
                    .y(6.0)
                    .size((width - 180.0).max(0.0), 15.0)
                    .text(&preview.song_name)
                    .text_justify(nuon::TextJustify::Left)
                    .build(ui);

                nuon::label()
                    .x(40.0)
                    .y(21.0)
                    .size((width - 180.0).max(0.0), 15.0)
                    .text(format!(
                        "{:.1}s / {:.1}s",
                        preview.current_time, preview.length
                    ))
                    .text_justify(nuon::TextJustify::Left)
                    .build(ui);

                nuon::translate().x(width).build(ui, |ui| {
                    if preview.available {
                        if Self::preview_bar_button()
                            .x(-30.0)
                            .icon(if preview.is_paused {
                                icons::play_icon()
                            } else {
                                icons::pause_icon()
                            })
                            .build(ui)
                        {
                            toggle_play = true;
                        }
                    } else {
                        nuon::quad()
                            .x(-30.0)
                            .size(30.0, 30.0)
                            .color([52, 52, 52])
                            .border_radius([5.0; 4])
                            .build(ui);
                        nuon::label()
                            .x(-30.0)
                            .size(30.0, 30.0)
                            .icon(icons::play_icon())
                            .color([140, 140, 140])
                            .build(ui);
                    }

                    if nuon::button()
                        .x(-85.0)
                        .size(45.0, 30.0)
                        .border_radius([5.0; 4])
                        .color([67, 67, 67])
                        .hover_color([87, 87, 87])
                        .preseed_color([97, 97, 97])
                        .label("SAVE")
                        .build(ui)
                    {
                        save_clicked = true;
                    }

                    if nuon::button()
                        .x(-147.0)
                        .size(52.0, 30.0)
                        .border_radius([5.0; 4])
                        .label(record_label)
                        .color(if self.recorder.is_recording() {
                            [125, 27, 27]
                        } else {
                            [67, 67, 67]
                        })
                        .hover_color(if self.recorder.is_recording() {
                            [145, 37, 37]
                        } else {
                            [87, 87, 87]
                        })
                        .preseed_color(if self.recorder.is_recording() {
                            [165, 47, 47]
                        } else {
                            [97, 97, 97]
                        })
                        .build(ui)
                    {
                        record_clicked = true;
                    }
                });

                nuon::translate().y(30.0).build(ui, |ui| {
                    if preview.available {
                        let event = nuon::click_area("FreeplayPreviewProgress")
                            .size(width, 45.0)
                            .build(ui);

                        if event.is_pressed() {
                            seek = true;
                        }
                    }

                    nuon::quad().size(width, 45.0).color([58, 58, 58]).build(ui);
                    nuon::quad()
                        .size(width * preview.progress, 45.0)
                        .color([56, 145, 255])
                        .build(ui);

                    if preview.length > 0.0 {
                        for measure in preview.measures.iter() {
                            let x = (measure.as_secs_f32() / preview.length) * width;
                            let color = if x < width * preview.progress {
                                nuon::Color::new(1.0, 1.0, 1.0, 0.5)
                            } else {
                                nuon::Color::new(0.4, 0.4, 0.4, 1.0)
                            };

                            nuon::quad().x(x).size(1.0, 45.0).color(color).build(ui);
                        }
                    }

                    nuon::label()
                        .x(10.0)
                        .y(15.0)
                        .size((width - 20.0).max(0.0), 15.0)
                        .text(&status_label)
                        .text_justify(nuon::TextJustify::Left)
                        .color(if preview.available {
                            [210, 210, 210]
                        } else {
                            [170, 170, 170]
                        })
                        .build(ui);
                });
            });

        if go_back {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(self.song.clone()))
                .ok();
        }

        if toggle_play && preview.available {
            self.toggle_preview_playback();
        }

        if seek && preview.available {
            self.seek_preview_to_cursor(ctx);
        }

        if record_clicked {
            self.handle_record_click(ctx);
        }

        if save_clicked {
            self.handle_save_click(ctx);
        }
    }

    fn update_glow(&mut self, delta: Duration) {
        let Some(glow) = &mut self.glow else {
            return;
        };

        glow.clear();

        let keys = &self.keyboard.layout().keys;
        let states = self.keyboard.key_states();

        for (key, state) in keys.iter().zip(states) {
            let Some(mut color) = state.pressed_by_user().copied() else {
                continue;
            };

            color.r *= 0.5;
            color.g *= 0.5;
            color.b *= 0.5;

            glow.push(
                key.id(),
                color,
                key.x(),
                self.keyboard.pos().y,
                key.width(),
                delta,
            );
        }
    }

    fn update_ui(&mut self, ctx: &mut Context) {
        self.update_preview_ui(ctx);

        nuon::label()
            .text(&self.deduced_chord_name)
            .font_size(25.0)
            .y(self.keyboard.pos().y - 25.0 - 10.0)
            .height(25.0)
            .width(ctx.window_state.logical_size.width)
            .build(&mut self.nuon);
    }

    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);
        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
        if let Some(note_labels) = self
            .preview_state
            .as_mut()
            .and_then(|state| state.note_labels.as_mut())
        {
            note_labels.set_pos(*self.keyboard.pos());
        }
        if let Some(waterfall) = self
            .preview_state
            .as_mut()
            .map(|state| &mut state.waterfall)
        {
            waterfall.resize(&ctx.config, self.keyboard.layout().clone());
        }
    }
}

impl Scene for FreeplayScene {
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

        let preview_time = self.update_preview(ctx, delta);

        if let Some(time) = preview_time
            && let Some(waterfall) = self
                .preview_state
                .as_mut()
                .map(|state| &mut state.waterfall)
        {
            waterfall.update(time);
        }

        let time = 0.0;

        self.guidelines.update(
            &mut self.quad_renderer_bg,
            ctx.config.animation_speed(),
            ctx.window_state.scale_factor as f32,
            time,
            ctx.window_state.logical_size,
        );
        self.keyboard
            .update(&mut self.quad_renderer_fg, &mut self.text_renderer);
        if let Some(time) = preview_time
            && let Some(note_labels) = self
                .preview_state
                .as_mut()
                .and_then(|state| state.note_labels.as_mut())
        {
            note_labels.update(
                ctx.window_state.physical_size,
                ctx.window_state.scale_factor as f32,
                self.keyboard.renderer(),
                ctx.config.animation_speed(),
                time,
            );
        }

        self.update_glow(delta);

        self.quad_renderer_bg.prepare();
        self.quad_renderer_fg.prepare();

        if let Some(glow) = &mut self.glow {
            glow.prepare();
        }

        self.text_renderer.update(
            ctx.window_state.physical_size,
            ctx.window_state.scale_factor as f32,
        );

        self.update_ui(ctx);

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);
    }

    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quad_renderer_bg.render(rpass);
        if let Some(preview_state) = self.preview_state.as_mut() {
            preview_state.waterfall.render(rpass);
            if let Some(note_labels) = preview_state.note_labels.as_mut() {
                note_labels.render(rpass);
            }
        }
        self.quad_renderer_fg.render(rpass);
        if let Some(glow) = &self.glow {
            glow.render(rpass);
        }
        self.text_renderer.render(rpass);
        self.nuon_renderer.render(rpass);
    }

    fn window_event(&mut self, ctx: &mut Context, event: &WindowEvent) {
        if event.window_resized() || event.scale_factor_changed() {
            self.resize(ctx)
        }

        if event.back_mouse_pressed() || event.key_released(Key::Named(NamedKey::Escape)) {
            ctx.proxy
                .send_event(NeothesiaEvent::MainMenu(self.song.clone()))
                .ok();
        }

        if event.key_released(Key::Named(NamedKey::Space)) && self.preview_state.is_some() {
            self.toggle_preview_playback();
        }

        super::handle_nuon_window_event(&mut self.nuon, event, ctx);
        super::handle_pc_keyboard_to_midi_event(ctx, event);
        super::handle_mouse_to_midi_event(
            &mut self.keyboard,
            &mut self.mouse_to_midi_state,
            ctx,
            event,
        );
    }

    fn midi_event(&mut self, ctx: &mut Context, channel: u8, message: &MidiMessage) {
        self.recorder.push_event(channel, *message);
        self.keyboard.user_midi_event(message);
        ctx.output_manager
            .connection()
            .midi_event(0.into(), *message);

        if let MidiMessage::NoteOn { .. } = message {
            let start = self.keyboard.layout().range.start();

            let notes: Vec<u8> = self
                .keyboard
                .key_states()
                .iter()
                .enumerate()
                .filter(|(_, state)| state.pressed_by_user().is_some())
                .map(|(id, _)| id as u8 + start)
                .collect();

            self.deduced_chord_name = chords::deduce_name(&notes);
        }
    }
}

mod chords {
    /// Get chord name based on notes, eg. Cmaj7
    pub fn deduce_name(midi_notes: &[u8]) -> String {
        if midi_notes.is_empty() {
            return "No notes".to_string();
        }

        if midi_notes.len() == 1 {
            return note_name(midi_notes[0]).to_string();
        }

        // Normalize notes to a single octave and sort
        let mut normalized: Vec<u8> = midi_notes.iter().map(|&n| n % 12).collect();
        normalized.sort_unstable();
        normalized.dedup();

        if normalized.is_empty() {
            return String::new();
        }

        // Try each note as potential root
        for i in 0..normalized.len() {
            let root = normalized[i];
            let intervals = get_intervals(&normalized, root);

            if let Some(chord_type) = match_chord_type(intervals) {
                return format!("{}{}", note_name(root), chord_type);
            }
        }

        String::new()
    }

    // TODO: This is clunky, we should change names based on the scale in use
    fn note_name(midi: u8) -> &'static str {
        const NAMES: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        NAMES[(midi % 12) as usize]
    }

    fn get_intervals(notes: &[u8], root: u8) -> Vec<u8> {
        notes
            .iter()
            .map(|&n| (n + 12 - root) % 12)
            .filter(|&i| i != 0)
            .collect()
    }

    fn match_chord_type(mut intervals: Vec<u8>) -> Option<&'static str> {
        intervals.sort_unstable();

        match intervals.as_slice() {
            // Triads
            [3, 7] => Some("m"),   // minor
            [4, 7] => Some("M"),   // major
            [3, 6] => Some("dim"), // diminished
            [4, 8] => Some("aug"), // augmented

            // Seventh chords
            [3, 7, 10] => Some("m7"),      // minor 7th
            [4, 7, 11] => Some("maj7"),    // major 7th
            [4, 7, 10] => Some("7"),       // dominant 7th
            [3, 6, 9] => Some("dim7"),     // diminished 7th
            [3, 6, 10] => Some("m7b5"),    // half-diminished
            [4, 8, 10] => Some("aug7"),    // augmented 7th
            [4, 8, 11] => Some("augmaj7"), // augmented major 7th
            [3, 7, 11] => Some("mmaj7"),   // minor major 7th

            // Sixth chords
            [4, 7, 9] => Some("6"),  // major 6th
            [3, 7, 9] => Some("m6"), // minor 6th

            // Suspended chords
            [2, 7] => Some("sus2"),      // suspended 2nd
            [5, 7] => Some("sus4"),      // suspended 4th
            [5, 7, 10] => Some("7sus4"), // dominant 7th suspended 4th

            // Extended chords (9ths)
            [2, 4, 7, 10] => Some("9"),    // dominant 9th
            [2, 4, 7, 11] => Some("maj9"), // major 9th
            [2, 3, 7, 10] => Some("m9"),   // minor 9th

            // Power chord
            [7] => Some("5"), // power chord

            _ => None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_major_chords() {
            assert_eq!(deduce_name(&[60, 64, 67]), "CM"); // C major
            assert_eq!(deduce_name(&[62, 66, 69]), "DM"); // D major
        }

        #[test]
        fn test_minor_chords() {
            assert_eq!(deduce_name(&[60, 63, 67]), "Cm"); // C minor
            assert_eq!(deduce_name(&[57, 60, 64]), "Am"); // A minor
        }

        #[test]
        fn test_seventh_chords() {
            assert_eq!(deduce_name(&[60, 64, 67, 71]), "Cmaj7"); // C major 7
            assert_eq!(deduce_name(&[60, 64, 67, 70]), "C7"); // C dominant 7
            assert_eq!(deduce_name(&[60, 63, 67, 70]), "Cm7"); // C minor 7
        }

        #[test]
        fn test_other_chords() {
            assert_eq!(deduce_name(&[60, 63, 66]), "Cdim"); // C diminished
            assert_eq!(deduce_name(&[60, 64, 68]), "Caug"); // C augmented
            assert_eq!(deduce_name(&[60, 65, 67]), "Csus4"); // C sus4
            assert_eq!(deduce_name(&[60, 67]), "C5"); // C power chord
        }

        #[test]
        fn test_edge_cases() {
            assert_eq!(deduce_name(&[]), "No notes");
            assert_eq!(deduce_name(&[60]), "C");
            // Multiple octaves should normalize
            assert_eq!(deduce_name(&[48, 64, 67, 72]), "CM");
        }
    }
}
