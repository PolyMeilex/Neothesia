//! User fingering choices persisted in a sidecar file next to the MIDI file.
//!
//! Only manual overrides and hand assignments are stored, automatic fingering is always
//! recomputed on load.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use midi_file::MidiNote;
use serde::{Deserialize, Serialize};

use super::HandAssignment;

/// Identifies a note in a way that does not depend on the in-memory note order
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteKey {
    pub track: usize,
    pub start_us: u64,
    pub note: u8,
}

impl NoteKey {
    pub fn new(note: &MidiNote) -> Self {
        Self {
            track: note.track_id,
            start_us: note.start.as_micros() as u64,
            note: note.note,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FingeringStore {
    midi_path: Option<PathBuf>,
    midi_hash: u64,
    pub hands: BTreeMap<usize, HandAssignment>,
    pub overrides: BTreeMap<NoteKey, u8>,
}

impl FingeringStore {
    /// Loads the stored fingering for the given MIDI file, or returns an empty store
    pub fn load(midi_path: &Path) -> Self {
        let midi_hash = std::fs::read(midi_path)
            .map(|bytes| fnv1a(&bytes))
            .unwrap_or_default();

        let mut store = Self {
            midi_path: Some(midi_path.to_owned()),
            midi_hash,
            ..Default::default()
        };

        let candidates = [sidecar_path(midi_path), fallback_path(midi_hash)];
        let Some(file) = candidates
            .into_iter()
            .flatten()
            .filter(|path| path.exists())
            .find_map(|path| read_file(&path))
        else {
            return store;
        };

        let FingeringFile::V1(file) = file;

        if file.midi_hash != midi_hash {
            log::warn!(
                "{} changed since the fingering was saved, some overrides may not apply",
                midi_path.display()
            );
        }

        store.hands = file.hands;
        store.overrides = file
            .overrides
            .into_iter()
            .filter(|o| (1..=5).contains(&o.finger))
            .map(|o| {
                let key = NoteKey {
                    track: o.track,
                    start_us: o.start_us,
                    note: o.note,
                };
                (key, o.finger)
            })
            .collect();

        store
    }

    /// Writes the sidecar file, falls back to the config directory if the MIDI file
    /// directory is read only. Does nothing for songs without a file (eg. recordings).
    pub fn save(&self) {
        let Some(midi_path) = self.midi_path.as_deref() else {
            return;
        };

        let file = FingeringFile::V1(FingeringFileV1 {
            midi_hash: self.midi_hash,
            hands: self.hands.clone(),
            overrides: self
                .overrides
                .iter()
                .map(|(key, finger)| FingerOverride {
                    track: key.track,
                    start_us: key.start_us,
                    note: key.note,
                    finger: *finger,
                })
                .collect(),
        });

        let s = match ron::ser::to_string_pretty(&file, ron::ser::PrettyConfig::default()) {
            Ok(s) => s,
            Err(err) => {
                log::error!("Failed to serialize fingering: {err}");
                return;
            }
        };

        if let Some(path) = sidecar_path(midi_path)
            && std::fs::write(&path, &s).is_ok()
        {
            return;
        }

        if let Some(path) = fallback_path(self.midi_hash) {
            std::fs::create_dir_all(path.parent().unwrap()).ok();
            if let Err(err) = std::fs::write(&path, &s) {
                log::error!("Failed to save fingering to {}: {err}", path.display());
            }
        }
    }

    pub fn set_override(&mut self, key: NoteKey, finger: Option<u8>) {
        match finger {
            Some(finger) => self.overrides.insert(key, finger),
            None => self.overrides.remove(&key),
        };
    }

    pub fn hand(&self, track_id: usize) -> HandAssignment {
        self.hands.get(&track_id).copied().unwrap_or_default()
    }

    pub fn set_hand(&mut self, track_id: usize, hand: HandAssignment) {
        if hand == HandAssignment::Auto {
            self.hands.remove(&track_id);
        } else {
            self.hands.insert(track_id, hand);
        }
    }
}

#[derive(Serialize, Deserialize)]
enum FingeringFile {
    V1(FingeringFileV1),
}

#[derive(Serialize, Deserialize)]
struct FingeringFileV1 {
    midi_hash: u64,
    #[serde(default)]
    hands: BTreeMap<usize, HandAssignment>,
    #[serde(default)]
    overrides: Vec<FingerOverride>,
}

#[derive(Serialize, Deserialize)]
struct FingerOverride {
    track: usize,
    start_us: u64,
    note: u8,
    finger: u8,
}

fn read_file(path: &Path) -> Option<FingeringFile> {
    let s = std::fs::read_to_string(path).ok()?;
    match ron::from_str(&s) {
        Ok(file) => Some(file),
        Err(err) => {
            log::error!("Failed to parse {}: {err}", path.display());
            None
        }
    }
}

/// `song.mid` -> `song.mid.fingering.ron`
fn sidecar_path(midi_path: &Path) -> Option<PathBuf> {
    let mut name = midi_path.file_name()?.to_owned();
    name.push(".fingering.ron");
    Some(midi_path.with_file_name(name))
}

fn fallback_path(midi_hash: u64) -> Option<PathBuf> {
    let settings = crate::utils::resources::settings_ron()?;
    Some(
        settings
            .parent()?
            .join("fingering")
            .join(format!("{midi_hash:016x}.ron")),
    )
}

/// Stable across Rust versions, unlike `DefaultHasher`
fn fnv1a(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325, |hash, &b| {
        (hash ^ b as u64).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidecar_name() {
        assert_eq!(
            sidecar_path(Path::new("/music/song.mid")).unwrap(),
            Path::new("/music/song.mid.fingering.ron")
        );
    }

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir().join(format!("neothesia-fingering-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let midi = dir.join("song.mid");
        std::fs::write(&midi, b"MThd").unwrap();

        let mut store = FingeringStore::load(&midi);
        assert!(store.overrides.is_empty());

        let key = NoteKey {
            track: 1,
            start_us: 1_500_000,
            note: 60,
        };
        store.set_override(key, Some(3));
        store.set_hand(1, HandAssignment::Left);
        store.save();

        let loaded = FingeringStore::load(&midi);
        assert_eq!(loaded.overrides.get(&key), Some(&3));
        assert_eq!(loaded.hand(1), HandAssignment::Left);
        assert_eq!(loaded.hand(2), HandAssignment::Auto);

        std::fs::remove_dir_all(&dir).ok();
    }
}
