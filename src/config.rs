use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub speed_multiplier: f32,
    pub playback_offset: f32,
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
            speed_multiplier: 1.0,
            playback_offset: 0.0,
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
