use std::time::Duration;

use midi_file::midly::MidiMessage;
use neothesia_core::render::{GlowRenderer, GuidelineRenderer, QuadRenderer, TextRenderer};
use winit::{
    event::WindowEvent,
    keyboard::{Key, NamedKey},
};

use crate::{
    NeothesiaEvent,
    context::Context,
    scene::{MouseToMidiEventState, NuonRenderer, Scene, playing_scene::Keyboard},
    song::Song,
    utils::window::WinitEvent,
};

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

    // SoundFont management
    soundfonts: Vec<crate::output_manager::SoundFontEntry>,
    current_soundfont_index: usize,

    // Gain button hold state
    gain_decrease_held: bool,
    gain_increase_held: bool,
    gain_hold_timer: f32,
}

impl FreeplayScene {
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

        // Discover SoundFonts from config
        let soundfont_folders = ctx.config.synth_config.soundfont_folders().clone();
        let soundfonts = crate::output_manager::discover_soundfonts(&soundfont_folders);

        // Determine current index from config or default to 0
        let current_soundfont_index = ctx.config.synth_config.soundfont_index().unwrap_or(0);

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
            soundfonts,
            current_soundfont_index,
            gain_decrease_held: false,
            gain_increase_held: false,
            gain_hold_timer: 0.0,
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
        let mut ui = std::mem::replace(&mut self.nuon, nuon::Ui::new());
        let window_size = ctx.window_state.logical_size;

        // Render ribbon background (always visible)
        nuon::quad()
            .size(window_size.width, 40.0)
            .color([37, 35, 42])
            .build(&mut ui);
        
        // Left panel: Back button
        nuon::translate()
            .x(10.0)
            .y(5.0)
            .build(&mut ui, |ui| {
                if nuon::button()
                    .size(30.0, 30.0)
                    .border_radius([5.0; 4])
                    .color([67, 67, 67])
                    .hover_color([87, 87, 87])
                    .preseed_color([97, 97, 97])
                    .icon(crate::icons::left_arrow_icon())
                    .build(ui)
                {
                    ctx.proxy
                        .send_event(NeothesiaEvent::MainMenu(self.song.clone()))
                        .ok();
                }
            });
        
        // Center panel: SoundFont controls
        let soundfont_name = self.current_soundfont_name();
        nuon::translate()
            .x(window_size.width / 2.0 - 200.0)
            .y(10.0)
            .build(&mut ui, |ui| {
                nuon::label()
                    .text(soundfont_name)
                    .size(400.0, 20.0)
                    .build(ui);
            });
        
        nuon::translate()
            .x(window_size.width / 2.0 - 250.0)
            .y(5.0)
            .build(&mut ui, |ui| {
                if nuon::button()
                    .size(30.0, 30.0)
                    .border_radius([5.0; 4])
                    .color([67, 67, 67])
                    .hover_color([87, 87, 87])
                    .preseed_color([97, 97, 97])
                    .icon(crate::icons::left_arrow_icon())
                    .build(ui)
                {
                    self.previous_soundfont(ctx);
                }
            });
        
        nuon::translate()
            .x(window_size.width / 2.0 + 220.0)
            .y(5.0)
            .build(&mut ui, |ui| {
                if nuon::button()
                    .size(30.0, 30.0)
                    .border_radius([5.0; 4])
                    .color([67, 67, 67])
                    .hover_color([87, 87, 87])
                    .preseed_color([97, 97, 97])
                    .icon(crate::icons::right_arrow_icon())
                    .build(ui)
                {
                    self.next_soundfont(ctx);
                }
            });
        
        // Right panel: Audio Gain controls
        let gain = ctx.config.synth_config.audio_gain();
        nuon::translate()
            .x(window_size.width - 180.0)
            .y(10.0)
            .build(&mut ui, |ui| {
                nuon::label()
                    .text(format!("Gain: {:.1}", gain))
                    .size(80.0, 20.0)
                    .build(ui);
            });
        
        nuon::translate()
            .x(window_size.width - 100.0)
            .y(5.0)
            .build(&mut ui, |ui| {
                let res = nuon::click_area("gain_decrease")
                    .size(30.0, 30.0)
                    .build(ui);

                let color = if res.is_pressed() {
                    [97, 97, 97]
                } else if res.is_hovered() {
                    [87, 87, 87]
                } else {
                    [67, 67, 67]
                };

                nuon::quad()
                    .size(30.0, 30.0)
                    .border_radius([5.0; 4])
                    .color(color)
                    .build(ui);

                nuon::label()
                    .icon(crate::icons::minus_icon())
                    .size(30.0, 30.0)
                    .build(ui);

                if res.is_press_start() {
                    self.gain_decrease_held = true;
                    self.gain_hold_timer = 0.1; // Trigger immediately
                    self.decrease_audio_gain(ctx);
                } else if res.is_clicked() && !self.gain_decrease_held {
                    self.decrease_audio_gain(ctx);
                }

                if !res.is_pressed() && self.gain_decrease_held {
                    self.gain_decrease_held = false;
                    self.gain_hold_timer = 0.0;
                }
            });
        
        nuon::translate()
            .x(window_size.width - 65.0)
            .y(5.0)
            .build(&mut ui, |ui| {
                let res = nuon::click_area("gain_increase")
                    .size(30.0, 30.0)
                    .build(ui);

                let color = if res.is_pressed() {
                    [97, 97, 97]
                } else if res.is_hovered() {
                    [87, 87, 87]
                } else {
                    [67, 67, 67]
                };

                nuon::quad()
                    .size(30.0, 30.0)
                    .border_radius([5.0; 4])
                    .color(color)
                    .build(ui);

                nuon::label()
                    .icon(crate::icons::plus_icon())
                    .size(30.0, 30.0)
                    .build(ui);

                if res.is_press_start() {
                    self.gain_increase_held = true;
                    self.gain_hold_timer = 0.1; // Trigger immediately
                    self.increase_audio_gain(ctx);
                } else if res.is_clicked() && !self.gain_increase_held {
                    self.increase_audio_gain(ctx);
                }

                if !res.is_pressed() && self.gain_increase_held {
                    self.gain_increase_held = false;
                    self.gain_hold_timer = 0.0;
                }
            });

        self.nuon = ui;
    }

    /// Get the current SoundFont name for display
    fn current_soundfont_name(&self) -> String {
        if let Some(entry) = self.soundfonts.get(self.current_soundfont_index) {
            let file_name = entry.path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            
            let folder_name = entry.folder.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown");
            
            let count = self.soundfonts.len();
            if count > 0 {
                format!("{} ({} of {}) from [{}]",
                    file_name,
                    self.current_soundfont_index + 1,
                    count,
                    folder_name
                )
            } else {
                file_name.to_string()
            }
        } else {
            "No SoundFont".to_string()
        }
    }

    /// Switch to previous SoundFont
    fn previous_soundfont(&mut self, ctx: &mut Context) {
        if self.soundfonts.is_empty() {
            return;
        }
        
        let count = self.soundfonts.len();
        let new_index = if self.current_soundfont_index == 0 {
            count - 1
        } else {
            self.current_soundfont_index - 1
        };
        
        self.switch_to_soundfont_index(new_index, ctx);
    }

    /// Switch to next SoundFont
    fn next_soundfont(&mut self, ctx: &mut Context) {
        if self.soundfonts.is_empty() {
            return;
        }
        
        let count = self.soundfonts.len();
        let new_index = (self.current_soundfont_index + 1) % count;
        self.switch_to_soundfont_index(new_index, ctx);
    }

    /// Switch to a specific SoundFont by index
    fn switch_to_soundfont_index(&mut self, index: usize, ctx: &mut Context) {
        if let Some(entry) = self.soundfonts.get(index) {
            self.current_soundfont_index = index;
            
            // Use the existing switch_soundfont method for hot-swapping
            if let Err(e) = ctx.output_manager.switch_soundfont(&entry.path) {
                eprintln!("Failed to switch SoundFont: {}", e);
            }
            
            // Update config to persist the selection
            ctx.config.synth_config.set_soundfont_path(Some(entry.path.clone()));
            ctx.config.synth_config.set_soundfont_index(Some(index));
            
            // Save config
            ctx.config.save();
        }
    }

    /// Decrease audio gain
    fn decrease_audio_gain(&mut self, ctx: &mut Context) {
        let current = ctx.config.synth_config.audio_gain();
        let new_gain = (current - 0.1).max(0.0);
        ctx.config.synth_config.set_audio_gain(new_gain);
        ctx.output_manager.connection().set_gain(new_gain);
        let _ = ctx.config.save();
    }

    /// Increase audio gain
    fn increase_audio_gain(&mut self, ctx: &mut Context) {
        let current = ctx.config.synth_config.audio_gain();
        let new_gain = current + 0.1;
        ctx.config.synth_config.set_audio_gain(new_gain);
        ctx.output_manager.connection().set_gain(new_gain);
        let _ = ctx.config.save();
    }

    fn resize(&mut self, ctx: &mut Context) {
        self.keyboard.resize(ctx);
        self.guidelines.set_layout(self.keyboard.layout().clone());
        self.guidelines.set_pos(*self.keyboard.pos());
    }
}

impl Scene for FreeplayScene {
    fn update(&mut self, ctx: &mut Context, delta: Duration) {
        self.quad_renderer_bg.clear();
        self.quad_renderer_fg.clear();

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

        // Handle continuous hold for gain buttons
        const GAIN_HOLD_DELAY: f32 = 0.1; // Seconds between increments when held
        let delta_secs = delta.as_secs_f32();

        if self.gain_decrease_held {
            self.gain_hold_timer += delta_secs;
            if self.gain_hold_timer >= GAIN_HOLD_DELAY {
                self.gain_hold_timer = 0.0;
                self.decrease_audio_gain(ctx);
            }
        } else if self.gain_increase_held {
            self.gain_hold_timer += delta_secs;
            if self.gain_hold_timer >= GAIN_HOLD_DELAY {
                self.gain_hold_timer = 0.0;
                self.increase_audio_gain(ctx);
            }
        }

        super::render_nuon(&mut self.nuon, &mut self.nuon_renderer, ctx);
    }

    fn render<'pass>(&'pass mut self, rpass: &mut wgpu_jumpstart::RenderPass<'pass>) {
        self.quad_renderer_bg.render(rpass);
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

        super::handle_nuon_window_event(&mut self.nuon, event, ctx);
        super::handle_pc_keyboard_to_midi_event(ctx, event);
        super::handle_mouse_to_midi_event(
            &mut self.keyboard,
            &mut self.mouse_to_midi_state,
            ctx,
            event,
        );
    }

    fn midi_event(&mut self, ctx: &mut Context, _channel: u8, message: &MidiMessage) {
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
