use crate::{program_track::ProgramTrack, tempo_track::TempoTrack, MidiTrack};
use midly::{Format, Smf, Timing};
use std::{fs, path::Path, sync::Arc};

#[derive(Debug, Clone)]
pub struct MidiFile {
    pub format: Format,
    pub tracks: Arc<[MidiTrack]>,
    pub program_map: ProgramTrack,
}

impl MidiFile {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let data = match fs::read(path) {
            Ok(buff) => buff,
            Err(_) => return Err(String::from("Could Not Open File")),
        };

        let smf = match Smf::parse(&data) {
            Ok(smf) => smf,
            Err(_) => return Err(String::from("Midi Parsing Error (midly lib)")),
        };

        let u_per_quarter_note: u16 = match smf.header.timing {
            Timing::Metrical(t) => t.as_int(),
            Timing::Timecode(_fps, _u) => {
                return Err(String::from("Midi With Timecode Timing, Not Supported!"));
            }
        };

        if smf.tracks.is_empty() {
            return Err(String::from("Midi File Has No Tracks"));
        }

        let tempo_track = TempoTrack::build(&smf.tracks, u_per_quarter_note);

        let mut track_color_id = 0;
        let tracks: Vec<MidiTrack> = smf
            .tracks
            .iter()
            .enumerate()
            .map(|(id, events)| {
                let track = MidiTrack::new(id, track_color_id, &tempo_track, events);

                if !track.notes.is_empty() {
                    track_color_id += 1;
                }

                track
            })
            .collect();

        let program_map = ProgramTrack::new(&tracks);

        Ok(Self {
            format: smf.header.format,
            tracks: tracks.into(),
            program_map,
        })
    }
}
