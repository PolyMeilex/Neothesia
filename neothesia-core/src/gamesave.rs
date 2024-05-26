use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedStats {
    pub songs: Vec<SongStats>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    pub fn load_for_song(songname: String) -> Vec<SongStats> {
        if let Some(saved_stats) = SavedStats::load() {
            // Filter stats for the current song
            let filtered_stats: Vec<SongStats> = saved_stats
                .songs
                .iter()
                .filter(|stats| stats.song_name == songname)
                .cloned()
                .collect();

            // Sort stats using the defined scoring cooking logic
            let mut sorted_stats = filtered_stats.clone();

            sorted_stats.sort_by(|a, b| {
                let score_a = SavedStats::score_cooking(a);
                let score_b = SavedStats::score_cooking(b);
                score_b.cmp(&score_a)
            });

            sorted_stats
        } else {
            vec![]
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
    pub fn score_cooking(stats: &SongStats) -> u32 {
        let mut score = stats.notes_hit + stats.correct_note_times * 10;
        // Apply penalties then give the bonus
        score =
            score.saturating_sub(stats.notes_missed + stats.wrong_notes) + stats.correct_note_times;

        score
    }
}

impl Default for SavedStats {
    fn default() -> Self {
        SavedStats { songs: Vec::new() }
    }
}
