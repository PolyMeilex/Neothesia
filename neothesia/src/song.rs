use midi_file::MidiTrack;

#[derive(Debug, Clone)]
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
}

#[derive(Debug, Clone)]
pub struct SongConfig {
    pub tracks: Box<[TrackConfig]>,
}

impl SongConfig {
    fn new(tracks: &[MidiTrack]) -> Self {
        let tracks: Vec<_> = tracks
            .iter()
            .map(|t| {
                let is_drums = t.has_drums && !t.has_other_than_drums;
                TrackConfig {
                    track_id: t.track_id,
                    player: PlayerConfig::Auto,
                    visible: !is_drums,
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
}

impl Song {
    pub fn new(file: midi_file::MidiFile) -> Self {
        let config = SongConfig::new(&file.tracks);
        Self { file, config }
    }
}
