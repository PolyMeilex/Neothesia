use std::path::PathBuf;

mod model;

use model::{
    AppearanceConfig, AppearanceConfigV1, DevicesConfig, DevicesConfigV1, History, HistoryV1,
    LayoutConfig, LayoutConfigV1, Model, PcKeyboardConfig, PcKeyboardConfigV1, PlaybackConfig,
    PlaybackConfigV1, SynthConfig, SynthConfigV1, WaterfallConfig, WaterfallConfigV1,
};
pub use model::{ColorSchemaV1, NoteNaming};

impl NoteNaming {
    pub const ALL: [Self; 2] = [Self::Letters, Self::SolfegePt];

    pub fn note_name(self, note: u8) -> &'static str {
        const LETTERS: [&str; 12] = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        const SOLFEGE_PT: [&str; 12] = [
            "Dó", "Ré♭", "Ré", "Mi♭", "Mi", "Fá", "Sol♭", "Sol", "Lá♭", "Lá", "Si♭", "Si",
        ];

        let names = match self {
            Self::Letters => &LETTERS,
            Self::SolfegePt => &SOLFEGE_PT,
        };
        names[(note % 12) as usize]
    }
}

fn ron_options() -> ron::Options {
    ron::Options::default()
        .with_default_extension(ron::extensions::Extensions::UNWRAP_VARIANT_NEWTYPES)
}

impl Model {
    fn load() -> Self {
        let config: Option<Self> = if let Some(path) = crate::utils::resources::settings_ron() {
            if let Ok(file) = std::fs::read_to_string(path) {
                match ron_options().from_str(&file) {
                    Ok(config) => Some(config),
                    Err(err) => {
                        log::error!("{err:#?}");
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        config.unwrap_or_default()
    }

    fn from_config(config: Config) -> Self {
        let Config {
            playback,
            waterfall,
            devices,
            history,
            synth,
            keyboard_layout,
            appearance,
            pc_keyboard,
        } = config;

        Self {
            waterfall: WaterfallConfig::V1(waterfall),
            playback: PlaybackConfig::V1(playback),
            history: History::V1(history),
            synth: SynthConfig::V1(synth),
            keyboard_layout: LayoutConfig::V1(keyboard_layout),
            devices: DevicesConfig::V1(devices),
            appearance: AppearanceConfig::V1(appearance),
            pc_keyboard: PcKeyboardConfig::V1(pc_keyboard),
        }
    }

    fn build(self) -> Config {
        Config {
            playback: match self.playback {
                PlaybackConfig::V1(v) => v,
            },
            waterfall: match self.waterfall {
                WaterfallConfig::V1(v) => v,
            },
            appearance: match self.appearance {
                AppearanceConfig::V1(v) => v,
            },
            devices: match self.devices {
                DevicesConfig::V1(v) => v,
            },
            synth: match self.synth {
                SynthConfig::V1(v) => v,
            },
            history: match self.history {
                History::V1(v) => v,
            },
            keyboard_layout: match self.keyboard_layout {
                LayoutConfig::V1(v) => v,
            },
            pc_keyboard: match self.pc_keyboard {
                PcKeyboardConfig::V1(v) => v,
            },
        }
    }
}

#[derive(Clone)]
pub struct Config {
    playback: PlaybackConfigV1,
    waterfall: WaterfallConfigV1,
    appearance: AppearanceConfigV1,
    devices: DevicesConfigV1,
    synth: SynthConfigV1,
    history: HistoryV1,
    keyboard_layout: LayoutConfigV1,
    pc_keyboard: PcKeyboardConfigV1,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Model::load().build()
    }

    pub fn piano_range(&self) -> std::ops::RangeInclusive<u8> {
        self.keyboard_layout.range.0..=self.keyboard_layout.range.1
    }

    pub fn set_piano_range_start(&mut self, start: u8) {
        self.keyboard_layout.range.0 = start;
    }

    pub fn set_piano_range_end(&mut self, start: u8) {
        self.keyboard_layout.range.1 = start.min(127);
    }

    pub fn set_separate_channels(&mut self, separate_channels: bool) {
        self.devices.separate_channels = separate_channels;
    }

    pub fn separate_channels(&self) -> bool {
        self.devices.separate_channels
    }

    pub fn vertical_guidelines(&self) -> bool {
        self.appearance.vertical_guidelines
    }

    pub fn horizontal_guidelines(&self) -> bool {
        self.appearance.horizontal_guidelines
    }

    pub fn set_vertical_guidelines(&mut self, vertical_guidelines: bool) {
        self.appearance.vertical_guidelines = vertical_guidelines;
    }

    pub fn set_horizontal_guidelines(&mut self, horizontal_guidelines: bool) {
        self.appearance.horizontal_guidelines = horizontal_guidelines;
    }

    pub fn glow(&self) -> bool {
        self.appearance.glow
    }

    pub fn set_glow(&mut self, glow: bool) {
        self.appearance.glow = glow;
    }

    pub fn chord_identifier(&self) -> bool {
        self.appearance.chord_identifier
    }

    pub fn set_chord_identifier(&mut self, chord_identifier: bool) {
        self.appearance.chord_identifier = chord_identifier;
    }

    pub fn note_naming(&self) -> NoteNaming {
        self.appearance.note_naming
    }

    pub fn set_note_naming(&mut self, note_naming: NoteNaming) {
        self.appearance.note_naming = note_naming;
    }

    pub fn last_opened_song(&self) -> Option<&PathBuf> {
        self.history.last_opened_song.as_ref()
    }

    pub fn set_last_opened_song(&mut self, last_opened_song: Option<PathBuf>) {
        self.history.last_opened_song = last_opened_song;
    }

    pub fn soundfont_path(&self) -> Option<PathBuf> {
        self.synth
            .soundfont_path
            .as_ref()
            .cloned()
            .or_else(crate::utils::resources::default_sf2)
    }

    pub fn set_soundfont_path(&mut self, soundfont_path: Option<PathBuf>) {
        self.synth.soundfont_path = soundfont_path;
    }

    pub fn output(&self) -> Option<&str> {
        self.devices.output.as_deref()
    }

    pub fn set_output(&mut self, output: Option<String>) {
        self.devices.output = output;
    }

    pub fn input(&self) -> Option<&str> {
        self.devices.input.as_deref()
    }

    pub fn set_input<D: std::fmt::Display>(&mut self, v: Option<D>) {
        self.devices.input = v.map(|v| v.to_string());
    }

    pub fn background_color(&self) -> (u8, u8, u8) {
        self.appearance.background_color
    }

    pub fn set_background_color(&mut self, background_color: (u8, u8, u8)) {
        self.appearance.background_color = background_color;
    }

    pub fn color_schema(&self) -> &[ColorSchemaV1] {
        &self.appearance.color_schema
    }

    pub fn set_color_schema(&mut self, color_schema: Vec<ColorSchemaV1>) {
        self.appearance.color_schema = color_schema;
    }

    pub fn audio_gain(&self) -> f32 {
        self.synth.audio_gain
    }

    pub fn set_audio_gain(&mut self, gain: f32) {
        self.synth.audio_gain = gain.max(0.0);
    }

    pub fn animation_offset(&self) -> f32 {
        self.waterfall.animation_offset
    }

    pub fn set_animation_offset(&mut self, offset: f32) {
        self.waterfall.animation_offset = offset;
    }

    pub fn animation_speed(&self) -> f32 {
        self.waterfall.animation_speed
    }

    pub fn set_animation_speed(&mut self, speed: f32) {
        if speed == 0.0 {
            // 0.0 is invalid speed, let's skip it and negate
            self.waterfall.animation_speed = -self.waterfall.animation_speed;
        } else {
            self.waterfall.animation_speed = speed;
        }
    }

    pub fn set_note_labels(&mut self, show: bool) {
        self.waterfall.note_labels = show;
    }

    pub fn note_labels(&self) -> bool {
        self.waterfall.note_labels
    }

    pub fn speed_multiplier(&self) -> f32 {
        self.playback.speed_multiplier
    }

    pub fn set_speed_multiplier(&mut self, speed_multiplier: f32) {
        self.playback.speed_multiplier = speed_multiplier.max(0.0);
    }

    pub fn pc_keyboard_octave(&self) -> u8 {
        self.pc_keyboard.octave_shift.min(10)
    }

    pub fn pc_keyboard_shift_octave_up(&mut self) {
        self.pc_keyboard.octave_shift += 1;
    }

    pub fn pc_keyboard_shift_octave_down(&mut self) {
        if self.pc_keyboard.octave_shift == 0 {
            return;
        }
        self.pc_keyboard.octave_shift -= 1;
    }

    pub fn save(&self) {
        let res = ron_options().to_string_pretty(
            &Model::from_config(self.clone()),
            ron::ser::PrettyConfig::default(),
        );

        if let Ok(s) = res
            && let Some(path) = crate::utils::resources::settings_ron()
        {
            std::fs::create_dir_all(path.parent().unwrap()).ok();
            std::fs::write(path, s).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SETTINGS_V0_5_1: &str = r#"(
    waterfall: V1(
        animation_speed: 400.0,
        animation_offset: 0.0,
        note_labels: true,
    ),
    history: V1(
        last_opened_song: Some("/Users/me/Music/song.mid"),
    ),
    keyboard_layout: V1(
        range: (21, 108),
    ),
    devices: V1(
        output: Some("Buildin Synth"),
        input: Some("USB MIDI"),
        separate_channels: false,
    ),
    appearance: V1(
        background_color: (0, 0, 0),
        vertical_guidelines: true,
        horizontal_guidelines: true,
        glow: true,
        chord_identifier: false,
    ),
)"#;

    fn parse(ron: &str) -> Config {
        ron_options().from_str::<Model>(ron).unwrap().build()
    }

    #[test]
    fn settings_from_v0_5_1_still_load() {
        let config = parse(SETTINGS_V0_5_1);

        assert!(config.note_labels());
        assert_eq!(config.input(), Some("USB MIDI"));
        assert_eq!(config.note_naming(), NoteNaming::Letters);
    }

    #[test]
    fn unknown_note_naming_keeps_the_other_settings() {
        let ron = SETTINGS_V0_5_1.replace(
            "chord_identifier: false,",
            "chord_identifier: false, note_naming: Solfeggio,",
        );
        let config = parse(&ron);

        assert_eq!(config.note_naming(), NoteNaming::Letters);
        assert_eq!(config.input(), Some("USB MIDI"));
    }

    #[test]
    fn saved_settings_keep_note_naming() {
        let mut config = parse(SETTINGS_V0_5_1);
        config.set_note_naming(NoteNaming::SolfegePt);

        let saved = ron_options()
            .to_string_pretty(
                &Model::from_config(config),
                ron::ser::PrettyConfig::default(),
            )
            .unwrap();
        let config = parse(&saved);

        assert_eq!(config.note_naming(), NoteNaming::SolfegePt);
        assert_eq!(config.input(), Some("USB MIDI"));
    }

    #[test]
    fn letters_keep_the_existing_names() {
        let letters: Vec<_> = (60..72).map(|n| NoteNaming::Letters.note_name(n)).collect();
        let solfege: Vec<_> = (60..72)
            .map(|n| NoteNaming::SolfegePt.note_name(n))
            .collect();

        assert_eq!(
            letters,
            [
                "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"
            ]
        );
        assert_eq!(
            solfege,
            [
                "Dó", "Ré♭", "Ré", "Mi♭", "Mi", "Fá", "Sol♭", "Sol", "Lá♭", "Lá", "Si♭", "Si"
            ]
        );
    }
}
