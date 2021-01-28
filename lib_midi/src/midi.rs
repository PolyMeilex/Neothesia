use {
    crate::{MidiTrack, TracksParser},
    midly::{Format, Smf, Timing},
    std::fs,
};

#[derive(Debug, Clone)]
pub struct Midi {
    // pub tracks_count: u16,
    pub format: Format,
    pub tracks: Vec<MidiTrack>,
    pub merged_track: MidiTrack,
}

impl Midi {
    pub fn new(path: &str) -> Result<Self, String> {
        let data = match fs::read(path) {
            Ok(buff) => buff,
            Err(_) => return Err(String::from("Could Not Open File")),
        };

        let smf = match Smf::parse(&data) {
            Ok(smf) => smf,
            Err(_) => return Err(String::from("Midi Parsing Error (midly lib)")),
        };

        let u_per_quarter_note: u16;

        match smf.header.timing {
            Timing::Metrical(t) => u_per_quarter_note = t.as_int(),
            Timing::Timecode(_fps, _u) => {
                return Err(String::from("Midi With Timecode Timing, Not Supported!"));
            }
        };

        if smf.tracks.is_empty() {
            return Err(String::from("Midi File Has No Tracks"));
        }

        let mut tracks: Vec<MidiTrack> = Vec::new();
        for (i, trk) in smf.tracks.iter().enumerate() {
            tracks.push(MidiTrack::new(trk, i));
        }

        let tp = &mut TracksParser::new(u_per_quarter_note);

        match smf.header.format {
            Format::SingleTrack => {
                tp.parse(&mut tracks, &smf.tracks);
            }
            Format::Parallel => {
                tp.parse(&mut tracks, &smf.tracks);
            }
            Format::Sequential => {
                return Err(String::from("MultiSong Midi Not Supported"));
            }
        }

        let mut merged_track: MidiTrack = tracks[0].clone();

        for (i, trk) in tracks.iter().enumerate() {
            if i > 0 {
                for n in trk.notes.iter() {
                    let n = n.clone();
                    merged_track.notes.push(n);
                }
            }
        }

        // merged_track.notes = merged_track
        //     .notes
        //     .into_iter()
        //     .filter(|n| n.ch == 9)
        //     .collect();

        merged_track
            .notes
            .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

        // Asign Unique Id
        for (i, note) in merged_track.notes.iter_mut().enumerate() {
            note.id = i;
        }

        Ok(Self {
            // tracks_count: tracks.len() as u16,
            format: smf.header.format,
            tracks,
            merged_track,
        })
    }
}
