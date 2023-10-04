#[derive(Debug, Default, Clone)]
pub enum PlayerConfig {
    Mute,
    #[default]
    Auto,
    Human,
}

#[derive(Debug, Clone)]
pub struct TrackConfig {
    pub track_id: usize,
    pub player: PlayerConfig,
    pub visible: bool,
}

impl TrackConfig {
    fn new(track_id: usize) -> Self {
        Self {
            track_id,
            player: PlayerConfig::default(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SongConfig {
    pub tracks: Box<[TrackConfig]>,
}

impl SongConfig {
    fn new(tracks_count: usize) -> Self {
        let tracks: Vec<_> = (0..tracks_count).map(TrackConfig::new).collect();
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
        let config = SongConfig::new(file.tracks.len());
        Self { file, config }
    }
}
