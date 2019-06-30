pub mod event_parser;

mod track;
mod tracks_parser;
use track::MidiTrack;


pub struct Midi {
    pub tracks_count: u16,
    format: midly::Format,
    pub u_per_frame: u8,
    pub u_per_quarter_note: u16,
    frames_per_seconds: f32,
    pub tracks: Vec<MidiTrack>,
    pub merged_track: MidiTrack,
}

pub fn read_file(path: &str) -> Midi {
    let smf_buffer = midly::SmfBuffer::open(path).unwrap();
    let smf = smf_buffer.parse_collect().unwrap();

    let mut u_per_frame: u8 = 0;
    let mut u_per_quarter_note: u16 = 0;
    let mut frames_per_seconds: f32 = 0.0;

    match smf.header.timing {
        midly::Timing::Metrical(t) => u_per_quarter_note = t.as_int(),
        midly::Timing::Timecode(fps, u) => {
            u_per_frame = u;
            frames_per_seconds = fps.as_f32();
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
            for (i, track) in tracks.iter_mut().enumerate() {
                track.extract_notes(&smf.tracks[i], 21, 108);
            }
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
                n.track_id = i;
                merged_track.notes.push(n);
            }
        }
    }

    merged_track
        .notes
        .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

    Midi {
        tracks_count: tracks.len() as u16,
        format: smf.header.format,
        u_per_frame,
        u_per_quarter_note,
        frames_per_seconds,
        tracks: tracks,
        merged_track: merged_track,
    }
}
