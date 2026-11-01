use std::path::Path;

use midi_file::MidiTrack;
use neothesia_core::fingering::{FingeringStore, HandAssignment};

use crate::context::Context;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PlayerConfig {
    Mute,
    Auto,
    Human,
}

#[derive(Debug, Clone)]
pub struct TrackConfig {
    pub track_id: usize,
    pub player: PlayerConfig,
    pub visible: bool,
    pub hand: HandAssignment,
}

#[derive(Default, Debug, Clone)]
pub struct SongConfig {
    pub tracks: Box<[TrackConfig]>,
}

impl SongConfig {
    fn new(tracks: &[MidiTrack], fingering: &FingeringStore) -> Self {
        let tracks: Vec<_> = tracks
            .iter()
            .map(|t| {
                let is_drums = t.has_drums && !t.has_other_than_drums;
                TrackConfig {
                    track_id: t.track_id,
                    player: PlayerConfig::Auto,
                    visible: !is_drums,
                    hand: fingering.hand(t.track_id),
                }
            })
            .collect();
        Self {
            tracks: tracks.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Song {
    pub file: midi_file::MidiFile,
    pub config: SongConfig,
    /// User fingering choices, persisted next to the MIDI file
    pub fingering: FingeringStore,
}

impl Song {
    /// Song without a file on disk, fingering changes are not persisted
    pub fn new(file: midi_file::MidiFile) -> Self {
        Self::with_fingering(file, FingeringStore::default())
    }

    pub fn from_path(file: midi_file::MidiFile, path: &Path) -> Self {
        Self::with_fingering(file, FingeringStore::load(path))
    }

    fn with_fingering(file: midi_file::MidiFile, fingering: FingeringStore) -> Self {
        let config = SongConfig::new(&file.tracks, &fingering);
        Self {
            file,
            config,
            fingering,
        }
    }

    /// Parts that get fingering: visible, non-drum tracks with their hand assignment
    pub fn fingering_parts(&self) -> Vec<(usize, HandAssignment)> {
        self.config
            .tracks
            .iter()
            .filter(|t| t.visible)
            .filter(|t| {
                let track = &self.file.tracks[t.track_id];
                !(track.has_drums && !track.has_other_than_drums)
            })
            .map(|t| (t.track_id, t.hand))
            .collect()
    }

    pub fn from_env(ctx: &Context) -> Option<Self> {
        let path = std::env::args()
            .nth(1)
            .map(std::path::PathBuf::from)
            .or_else(|| ctx.config.last_opened_song().cloned())?;

        let midi_file = midi_file::MidiFile::new(&path).ok()?;
        Some(Self::from_path(midi_file, &path))
    }
}
