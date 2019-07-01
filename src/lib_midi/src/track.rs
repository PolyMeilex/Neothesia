use crate::tracks_parser::TracksParser;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TempoEvent {
    pub time_in_units: f64,
    pub tempo: u32,
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub start: f64,
    pub duration: f64,
    pub note: u8,
    pub vel: u8,
    pub ch: u8,
    pub track_id: usize,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
    pub tempo: u32,
    pub tempo_events: Vec<TempoEvent>,
    pub has_tempo: bool,
    pub notes: Vec<MidiNote>,
    pub track_id: usize,
}

impl MidiTrack {
    pub fn new(track: &Vec<midly::Event>, track_id: usize) -> MidiTrack {
        let mut tempo = 500000; // 120 bpm

        use midly::EventKind;
        use midly::MetaMessage;

        let mut has_tempo = false;
        let mut tempo_events = Vec::new();

        let mut time_in_units = 0.0;
        for event in track.iter() {
            time_in_units += event.delta.as_int() as f64;
            match &event.kind {
                EventKind::Meta(meta) => match &meta {
                    MetaMessage::Tempo(t) => {
                        if !has_tempo {
                            tempo = t.as_int();
                            has_tempo = true;
                        }
                        tempo_events.push(TempoEvent {
                            time_in_units,
                            tempo: t.as_int(),
                        });
                    }
                    _ => {}
                },
                _ => {}
            };
        }

        MidiTrack {
            tempo,
            tempo_events,
            has_tempo,
            track_id,
            notes: Vec::new(),
        }
    }

    pub fn extract_notes(&mut self, events: &Vec<midly::Event>, parent_parser: &mut TracksParser) {
        self.notes.clear();

        let mut time_in_units = 0.0;

        struct Note {
            time_in_units: f64,
            vel: u8,
            channel: u8,
        };
        let mut current_notes: HashMap<u8, Note> = HashMap::new();

        let mut index = 0;
        println!("New Track");
        for event in events.iter() {
            use midly::EventKind;
            use midly::MidiMessage;

            time_in_units += event.delta.as_int() as f64;
            println!("{}", time_in_units);
            match &event.kind {
                EventKind::Midi { channel, message } => {
                    match &message {
                        MidiMessage::NoteOn(data0, data1) => {
                            let data0 = data0.as_int();
                            let data1 = data1.as_int();
                            if data1 > 0 {
                                let k = data0;
                                current_notes.insert(
                                    k,
                                    Note {
                                        time_in_units: time_in_units,
                                        vel: data1,
                                        channel: channel.as_int(),
                                    },
                                );
                            } else if data1 == 0 {
                                let k = data0;

                                if current_notes.contains_key(&k) {
                                    let n = current_notes.get(&k).unwrap();

                                    let start =
                                        parent_parser.pulses_to_ms(n.time_in_units) / 1000.0;
                                    let duration =
                                        parent_parser.pulses_to_ms(time_in_units) / 1000.0 - start;

                                    let mn = MidiNote {
                                        start: start,
                                        duration: duration,
                                        note: k,
                                        vel: n.vel,
                                        ch: n.channel,
                                        track_id: self.track_id, // Placeholder
                                        id: index,
                                    };
                                    self.notes.push(mn);
                                }
                            }

                        }
                        MidiMessage::NoteOff(data0, _data1) => {
                            let data0 = data0.as_int();

                            let k = data0;

                            if current_notes.contains_key(&k) {
                                let n = current_notes.get(&k).unwrap();

                                let start = parent_parser.pulses_to_ms(n.time_in_units) / 1000.0;
                                let duration =
                                    parent_parser.pulses_to_ms(time_in_units) / 1000.0 - start;

                                let mn = MidiNote {
                                    start: start,
                                    duration: duration,
                                    note: k,
                                    vel: n.vel,
                                    ch: channel.as_int(),
                                    track_id: self.track_id, // Placeholder
                                    id: index,
                                };
                                self.notes.push(mn);
                            }

                        }
                        _ => {}
                    }
                }
                _ => {}
            }
            index += 1;
        }

        self.notes
            .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
    }
}
