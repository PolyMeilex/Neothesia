pub mod track;

mod tracks_parser;
use track::MidiTrack;


pub struct Midi {
    pub tracks_count: u16,
    pub format: midly::Format,
    pub tracks: Vec<MidiTrack>,
    pub merged_track: MidiTrack,
}

pub fn read_file(path: &str) -> Midi {
    let smf_buffer = midly::SmfBuffer::open(path).unwrap();
    let smf = smf_buffer.parse_collect().unwrap();

    // let mut u_per_quarter_note: u16 = 480;
    let u_per_quarter_note: u16;

    match smf.header.timing {
        midly::Timing::Metrical(t) => u_per_quarter_note = t.as_int(),
        midly::Timing::Timecode(_fps, _u) => {
            panic!("Midi With Timecode Timing, not supported!");
            // u_per_frame = u;
            // frames_per_seconds = fps.as_f32();
        }
    };

    if smf.tracks.len() == 0 {
        panic!("No Tracks!");
    }

    let mut tracks: Vec<MidiTrack> = Vec::new();
    for (i, trk) in smf.tracks.iter().enumerate() {
        tracks.push(MidiTrack::new(trk, i));
    }

    let tp = &mut tracks_parser::TracksParser::new(u_per_quarter_note);

    use midly::Format;
    match smf.header.format {
        Format::SingleTrack => {
            tp.parse(&mut tracks, &smf.tracks);
        }
        Format::Parallel => {
            tp.parse(&mut tracks, &smf.tracks);
        }
        Format::Sequential => {
            panic!("MultiSong Midi Not Supported");
        }
    }

    let mut merged_track: MidiTrack = tracks[0].clone();

    for (i, trk) in tracks.iter().enumerate() {
        if i > 0 {
            for n in trk.notes.iter() {
                let mut n = n.clone();
                merged_track.notes.push(n);
            }
        }
    }

    merged_track
        .notes
        .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    // Asign Unique Id
    for (i, note) in merged_track.notes.iter_mut().enumerate() {
        note.id = i;
    }

    Midi {
        tracks_count: tracks.len() as u16,
        format: smf.header.format,
        tracks: tracks,
        merged_track: merged_track,
    }
}
