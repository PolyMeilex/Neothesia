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

    #[serde(default = "default_note_labels")]
    pub note_labels: bool,
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
            note_labels: default_note_labels(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PlaybackConfigV1 {
    #[serde(default = "default_speed_multiplier")]
    pub speed_multiplier: f32,

    #[serde(default = "default_wait_mode")]
    pub wait_mode: bool,

    #[serde(default = "default_lumi_color_mode")]
    pub lumi_color_mode: u8,

    #[serde(default = "default_lumi_brightness")]
    pub lumi_brightness: u8,
}

#[derive(Serialize, Deserialize)]
pub enum PlaybackConfig {
    V1(PlaybackConfigV1),
}

impl Default for PlaybackConfig {
    fn default() -> Self {
        Self::V1(PlaybackConfigV1 {
            speed_multiplier: default_speed_multiplier(),
            wait_mode: default_wait_mode(),
            lumi_color_mode: default_lumi_color_mode(),
            lumi_brightness: default_lumi_brightness(),
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

#[derive(Serialize, Deserialize, Clone)]
pub struct SynthConfigV2 {
    pub soundfont_path: Option<PathBuf>,
    pub soundfont_folders: Vec<PathBuf>,
    pub soundfont_index: Option<usize>,
    #[serde(default = "default_audio_gain")]
    pub audio_gain: f32,
}

impl From<SynthConfigV1> for SynthConfigV2 {
    fn from(v1: SynthConfigV1) -> Self {
        Self {
            soundfont_path: v1.soundfont_path,
            soundfont_folders: Vec::new(),
            soundfont_index: None,
            audio_gain: v1.audio_gain,
        }
    }
}

impl Default for SynthConfigV2 {
    fn default() -> Self {
        Self {
            soundfont_path: None,
            soundfont_folders: Vec::new(),
            soundfont_index: None,
            audio_gain: default_audio_gain(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum SynthConfig {
    V1(SynthConfigV1),
    V2(SynthConfigV2),
}

impl Default for SynthConfig {
    fn default() -> Self {
        Self::V2(SynthConfigV2::default())
    }
}

impl SynthConfig {
    pub fn soundfont_path(&self) -> Option<&PathBuf> {
        match self {
            SynthConfig::V1(v1) => v1.soundfont_path.as_ref(),
            SynthConfig::V2(v2) => v2.soundfont_path.as_ref(),
        }
    }
    
    pub fn set_soundfont_path(&mut self, path: Option<PathBuf>) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                v2.soundfont_path = path;
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => v2.soundfont_path = path,
        }
    }
    
    pub fn soundfont_folders(&self) -> &Vec<PathBuf> {
        match self {
            SynthConfig::V1(_v1) => {
                static EMPTY: Vec<PathBuf> = Vec::new();
                &EMPTY
            }
            SynthConfig::V2(v2) => &v2.soundfont_folders,
        }
    }
    
    pub fn add_soundfont_folder(&mut self, folder: PathBuf) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                v2.soundfont_folders.push(folder);
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => v2.soundfont_folders.push(folder),
        }
    }
    
    pub fn remove_soundfont_folder(&mut self, index: usize) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                if index < v2.soundfont_folders.len() {
                    v2.soundfont_folders.remove(index);
                }
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => {
                if index < v2.soundfont_folders.len() {
                    v2.soundfont_folders.remove(index);
                }
            }
        }
    }
    
    pub fn clear_soundfont_folders(&mut self) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                v2.soundfont_folders.clear();
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => v2.soundfont_folders.clear(),
        }
    }
    
    pub fn set_soundfont_folders(&mut self, folders: Vec<PathBuf>) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                v2.soundfont_folders = folders;
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => v2.soundfont_folders = folders,
        }
    }
    
    pub fn soundfont_index(&self) -> Option<usize> {
        match self {
            SynthConfig::V1(_) => None,
            SynthConfig::V2(v2) => v2.soundfont_index,
        }
    }
    
    pub fn set_soundfont_index(&mut self, index: Option<usize>) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                v2.soundfont_index = index;
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => v2.soundfont_index = index,
        }
    }

    pub fn audio_gain(&self) -> f32 {
        match self {
            SynthConfig::V1(v1) => v1.audio_gain,
            SynthConfig::V2(v2) => v2.audio_gain,
        }
    }

    pub fn set_audio_gain(&mut self, gain: f32) {
        match self {
            SynthConfig::V1(v1) => {
                let mut v2 = SynthConfigV2::from(v1.clone());
                v2.audio_gain = gain;
                *self = SynthConfig::V2(v2);
            }
            SynthConfig::V2(v2) => v2.audio_gain = gain,
        }
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

    #[serde(default = "default_glow")]
    pub glow: bool,
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
            glow: default_glow(),
        })
    }
}

fn default_piano_range() -> (u8, u8) {
    (21, 108)
}

fn default_speed_multiplier() -> f32 {
    1.0
}

fn default_wait_mode() -> bool {
    false
}

fn default_lumi_color_mode() -> u8 {
    3 // Night mode
}

fn default_lumi_brightness() -> u8 {
    64 // 50%
}

fn default_animation_speed() -> f32 {
    400.0
}

fn default_animation_offset() -> f32 {
    0.0
}

fn default_note_labels() -> bool {
    false
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

fn default_glow() -> bool {
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
