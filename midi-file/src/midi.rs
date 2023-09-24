use crate::{program_track::ProgramTrack, tempo_track::TempoTrack, MidiEvent, MidiNote, MidiTrack};
use midly::{Format, Smf, Timing};
use std::{fs, path::Path, sync::Arc};

#[derive(Debug, Clone)]
pub struct MergedTracks {
    pub notes: Arc<[MidiNote]>,
    pub events: Arc<[MidiEvent]>,
}

#[derive(Debug, Clone)]
pub struct MidiFile {
    pub format: Format,
    pub tracks: Arc<[MidiTrack]>,
    pub merged_tracks: MergedTracks,
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

        let (notes_count, events_count) = tracks.iter().fold((0, 0), |(notes, events), track| {
            (notes + track.notes.len(), events + track.events.len())
        });

        let mut notes = Vec::with_capacity(notes_count);
        let mut events = Vec::with_capacity(events_count);

        for track in tracks.iter() {
            for n in track.notes.iter().cloned() {
                notes.push(n);
            }
            for e in track.events.iter().cloned() {
                events.push(e);
            }
        }

        notes.sort_by_key(|n| n.start);
        events.sort_by_key(|n| n.timestamp);

        let program_map = ProgramTrack::new(&events);

        let merged_track = MergedTracks {
            notes: notes.into(),
            events: events.into(),
        };

        Ok(Self {
            format: smf.header.format,
            tracks: tracks.into(),
            merged_tracks: merged_track,
            program_map,
        })
    }
}
