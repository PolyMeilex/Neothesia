use midi_file::MidiTrack;
use std::collections::HashSet;

use crate::context::Context;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelMode {
    Listen,
    Assist,
    Alone,
}

#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub channel: u8,
    pub mode: ChannelMode,
    pub active: bool,
    pub interactive: bool, // If false, channel plays but doesn't participate in Learn/wait mode
}

#[derive(Debug, Clone)]
pub struct TrackConfig {
    pub track_id: usize,
    pub channels: Vec<ChannelConfig>,
    pub visible: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlayMode {
    #[default]
    Watch,
    Learn,
    Play,
}

#[derive(Default, Debug, Clone)]
pub struct SongConfig {
    pub tracks: Box<[TrackConfig]>,
    pub wait_mode: bool,
    pub play_mode: PlayMode,
}

impl SongConfig {
    fn new(tracks: &[MidiTrack]) -> Self {
        let tracks: Vec<_> = tracks
            .iter()
            .map(|t| {
                let is_drums = t.has_drums && !t.has_other_than_drums;

                // Discover channels used by this track
                let used_channels: HashSet<u8> = t.notes.iter().map(|note| note.channel).collect();

                // Create ChannelConfig for each channel
                let channels: Vec<ChannelConfig> = used_channels
                    .into_iter()
                    .map(|channel| ChannelConfig {
                        channel,
                        mode: ChannelMode::Listen, // Default to Listen
                        active: true,
                        // Channel 9 (0-indexed) is the drum channel - not interactive in Learn mode
                        interactive: channel != 9,
                    })
                    .collect();

                TrackConfig {
                    track_id: t.track_id,
                    channels,
                    visible: !is_drums,
                }
            })
            .collect();
        Self {
            tracks: tracks.into(),
            wait_mode: false,
            play_mode: PlayMode::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Song {
    pub file: midi_file::MidiFile,
    pub config: SongConfig,
}

impl Song {
    pub fn new(file: midi_file::MidiFile) -> Self {
        let config = SongConfig::new(&file.tracks);
        Self { file, config }
    }

    pub fn from_env(ctx: &Context) -> Option<Self> {
        let args: Vec<String> = std::env::args().collect();
        let midi_file = if args.len() > 1 {
            midi_file::MidiFile::new(&args[1]).ok()
        } else if let Some(last) = ctx.config.last_opened_song() {
            midi_file::MidiFile::new(last).ok()
        } else {
            None
        };

        Some(Self::new(midi_file?))
    }
}
