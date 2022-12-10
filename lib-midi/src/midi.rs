use crate::{utils, MidiTrack};
use midly::{Format, Smf, Timing};
use std::{fs, path::Path};

#[derive(Debug, Clone)]
pub struct Midi {
    pub format: Format,
    pub tracks: Vec<MidiTrack>,
    pub merged_track: MidiTrack,
}

impl Midi {
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

        let tempo_track = utils::TempoTrack::build(&smf.tracks);

        let tracks: Vec<MidiTrack> = smf
            .tracks
            .iter()
            .enumerate()
            .map(|(id, events)| MidiTrack::new(id, &tempo_track, events, u_per_quarter_note))
            .collect();

        let mut merged_track: MidiTrack = tracks[0].clone();

        for track in tracks.iter().skip(1) {
            for n in track.notes.iter().cloned() {
                merged_track.notes.push(n);
            }
            for e in track.events.iter().cloned() {
                merged_track.events.push(e);
            }
        }

        merged_track.notes.sort_by_key(|n| n.start);
        merged_track.events.sort_by_key(|n| n.timestamp);

        // Assign Unique Id
        for (i, note) in merged_track.notes.iter_mut().enumerate() {
            note.id = i;
        }

        Ok(Self {
            format: smf.header.format,
            tracks,
            merged_track,
        })
    }
}
