use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_speed_multiplier")]
    pub speed_multiplier: f32,

    #[serde(default = "default_playback_offset")]
    pub playback_offset: f32,

    #[serde(default = "default_play_along")]
    pub play_along: bool,
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
