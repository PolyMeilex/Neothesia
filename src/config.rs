use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct ColorSchema {
    pub base: (u8, u8, u8),
    pub dark: (u8, u8, u8),
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_speed_multiplier")]
    pub speed_multiplier: f32,

    #[serde(default = "default_playback_offset")]
    pub playback_offset: f32,

    #[serde(default = "default_play_along")]
    #[serde(skip_serializing)]
    pub play_along: bool,

    #[serde(default = "default_color_schema")]
    pub color_schema: Vec<ColorSchema>,

    #[serde(default)]
    pub background_color: (u8, u8, u8),
}

impl Config {
    pub fn new() -> Self {
        let path = crate::resources::settings_ron();
        let config: Option<Config> = if let Ok(file) = std::fs::read_to_string(path) {
            match ron::from_str(&file) {
                Ok(config) => Some(config),
                Err(err) => {
                    log::error!("{:#?}", err);
                    None
                }
            }
        } else {
            None
        };

        config.unwrap_or_else(|| Self {
            speed_multiplier: default_speed_multiplier(),
            playback_offset: default_playback_offset(),
            play_along: default_play_along(),
            color_schema: default_color_schema(),
            background_color: Default::default(),
        })
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        if let Ok(s) = ron::ser::to_string_pretty(self, Default::default()) {
            let path = crate::resources::settings_ron();
            std::fs::write(path, &s).ok();
        }
    }
}

fn default_speed_multiplier() -> f32 {
    1.0
}

fn default_playback_offset() -> f32 {
    0.0
}

fn default_play_along() -> bool {
    false
}

fn default_color_schema() -> Vec<ColorSchema> {
    vec![
        ColorSchema {
            base: (93, 188, 255),
            dark: (48, 124, 255),
        },
        ColorSchema {
            base: (210, 89, 222),
            dark: (125, 69, 134),
        },
    ]
}
