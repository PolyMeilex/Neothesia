use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug)]
pub struct SavedStats {
    pub songs: Vec<SongStats>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SongStats {
    pub song_name: String,
    pub correct_note_times: u32,
    pub wrong_note_times: u32,
    pub notes_missed: u32,
    pub notes_hit: u32,
    pub wrong_notes: u32,
    pub date: SystemTime,
}

impl SavedStats {
    pub fn load() -> Option<SavedStats> {
        if let Some(path) = crate::utils::resources::gamestats_ron() {
            if let Ok(file) = std::fs::read_to_string(&path) {
                match ron::from_str(&file) {
                    Ok(stats) => Some(stats),
                    Err(err) => {
                        log::error!("Error loading game stats: {:#?}", err);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn save(&self) {
        if let Ok(s) = ron::ser::to_string_pretty(self, Default::default()) {
            if let Some(path) = crate::utils::resources::gamestats_ron() {
                std::fs::create_dir_all(path.parent().unwrap()).ok();
                std::fs::write(path, s).ok();
            }
        }
    }
}

impl Default for SavedStats {
    fn default() -> Self {
        SavedStats { songs: Vec::new() }
    }
}
