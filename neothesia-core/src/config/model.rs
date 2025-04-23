use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Model {
    #[serde(default)]
    pub waterfall: WaterfallConfig,
    #[serde(default)]
    pub playback: PlaybackConfig,
    #[serde(default)]
    pub history: History,
    #[serde(default)]
    pub synth: SynthConfig,
    #[serde(default)]
    pub keyboard_layout: LayoutConfig,
    #[serde(default)]
    pub devices: DevicesConfig,
    #[serde(default)]
    pub appearance: AppearanceConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WaterfallConfigV1 {
    #[serde(default = "default_animation_speed")]
    pub animation_speed: f32,

    #[serde(default = "default_animation_offset")]
    pub animation_offset: f32,
}

#[derive(Serialize, Deserialize)]
pub enum WaterfallConfig {
    V1(WaterfallConfigV1),
}

impl Default for WaterfallConfig {
    fn default() -> Self {
        Self::V1(WaterfallConfigV1 {
            animation_speed: default_animation_speed(),
            animation_offset: default_animation_offset(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaybackConfigV1 {
    #[serde(default = "default_speed_multiplier")]
    pub speed_multiplier: f32,
}

#[derive(Serialize, Deserialize)]
pub enum PlaybackConfig {
    V1(PlaybackConfigV1),
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self::V1(PlaybackConfigV1 {
            speed_multiplier: default_speed_multiplier(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct HistoryV1 {
    pub last_opened_song: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub enum History {
    V1(HistoryV1),
}

impl Default for History {
    fn default() -> Self {
        Self::V1(HistoryV1 {
            last_opened_song: None,
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SynthConfigV1 {
    pub soundfont_path: Option<PathBuf>,
    #[serde(default = "default_audio_gain")]
    pub audio_gain: f32,
}

#[derive(Serialize, Deserialize)]
pub enum SynthConfig {
    V1(SynthConfigV1),
}

impl Default for SynthConfig {
    fn default() -> Self {
        Self::V1(SynthConfigV1 {
            soundfont_path: None,
            audio_gain: default_audio_gain(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LayoutConfigV1 {
    #[serde(default = "default_piano_range")]
    pub range: (u8, u8),
}

#[derive(Serialize, Deserialize)]
pub enum LayoutConfig {
    V1(LayoutConfigV1),
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self::V1(LayoutConfigV1 {
            range: default_piano_range(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DevicesConfigV1 {
    #[serde(default = "default_output")]
    pub output: Option<String>,
    pub input: Option<String>,

    #[serde(default = "default_separate_channels")]
    pub separate_channels: bool,
}

#[derive(Serialize, Deserialize)]
pub enum DevicesConfig {
    V1(DevicesConfigV1),
}

impl Default for DevicesConfig {
    fn default() -> Self {
        Self::V1(DevicesConfigV1 {
            output: default_output(),
            input: None,
            separate_channels: default_separate_channels(),
        })
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct ColorSchemaV1 {
    pub base: (u8, u8, u8),
    pub dark: (u8, u8, u8),
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AppearanceConfigV1 {
    #[serde(default = "default_color_schema")]
    pub color_schema: Vec<ColorSchemaV1>,

    #[serde(default)]
    pub background_color: (u8, u8, u8),

    #[serde(default = "default_vertical_guidelines")]
    pub vertical_guidelines: bool,

    #[serde(default = "default_horizontal_guidelines")]
    pub horizontal_guidelines: bool,
}

#[derive(Serialize, Deserialize)]
pub enum AppearanceConfig {
    V1(AppearanceConfigV1),
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self::V1(AppearanceConfigV1 {
            color_schema: default_color_schema(),
            background_color: Default::default(),
            vertical_guidelines: default_vertical_guidelines(),
            horizontal_guidelines: default_horizontal_guidelines(),
        })
    }
}

fn default_piano_range() -> (u8, u8) {
    (21, 108)
}

fn default_speed_multiplier() -> f32 {
    1.0
}

fn default_animation_speed() -> f32 {
    400.0
}

fn default_animation_offset() -> f32 {
    0.0
}

fn default_audio_gain() -> f32 {
    0.2
}

fn default_vertical_guidelines() -> bool {
    true
}

fn default_horizontal_guidelines() -> bool {
    true
}

fn default_separate_channels() -> bool {
    false
}

fn default_color_schema() -> Vec<ColorSchemaV1> {
    vec![
        ColorSchemaV1 {
            base: (210, 89, 222),
            dark: (125, 69, 134),
        },
        ColorSchemaV1 {
            base: (93, 188, 255),
            dark: (48, 124, 255),
        },
        ColorSchemaV1 {
            base: (255, 126, 51),
            dark: (192, 73, 0),
        },
        ColorSchemaV1 {
            base: (51, 255, 102),
            dark: (0, 168, 2),
        },
        ColorSchemaV1 {
            base: (255, 51, 129),
            dark: (48, 124, 255),
        },
        ColorSchemaV1 {
            base: (210, 89, 222),
            dark: (125, 69, 134),
        },
    ]
}

fn default_output() -> Option<String> {
    Some("Buildin Synth".into())
}
