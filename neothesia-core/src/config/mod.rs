use std::path::PathBuf;

mod model;

pub use model::ColorSchemaV1;
use model::{
    AppearanceConfig, AppearanceConfigV1, DevicesConfig, DevicesConfigV1, History, HistoryV1,
    LayoutConfig, LayoutConfigV1, Model, PlaybackConfig, PlaybackConfigV1, SynthConfig,
    SynthConfigV1, WaterfallConfig, WaterfallConfigV1,
};

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
        } = config;

        Self {
            waterfall: WaterfallConfig::V1(waterfall),
            playback: PlaybackConfig::V1(playback),
            history: History::V1(history),
            synth: SynthConfig::V1(synth),
            keyboard_layout: LayoutConfig::V1(keyboard_layout),
            devices: DevicesConfig::V1(devices),
            appearance: AppearanceConfig::V1(appearance),
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

    pub fn last_opened_song(&self) -> Option<&PathBuf> {
        self.history.last_opened_song.as_ref()
    }

    pub fn set_last_opened_song(&mut self, last_opened_song: Option<PathBuf>) {
        self.history.last_opened_song = last_opened_song;
    }

    pub fn soundfont_path(&self) -> Option<&PathBuf> {
        self.synth.soundfont_path.as_ref()
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
