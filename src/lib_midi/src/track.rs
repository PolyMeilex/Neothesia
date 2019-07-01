// use crate::event_parser::MidiNote;
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
    // u_per_quarter_note: f64,
    // u_time: f64,
    // pub time_signature: f64,
    // pub seconds_per_measure: f64,
    pub has_tempo: bool,
    pub notes: Vec<MidiNote>,
    pub track_id: usize,
}

impl MidiTrack {
    pub fn new(track: &Vec<midly::Event>, track_id: usize) -> MidiTrack {
        let mut tempo = 500000; // 120 bpm
                                // let mut time_signature = 4.0 / 4.0;

        // let mut events: Vec<rimd::TrackEvent> = Vec::new();
        // let mut events: Vec<midly::Event> = track.to_owned(); // Clone For Later Use In Note Extraction

        use midly::EventKind;
        use midly::MetaMessage;

        let mut has_tempo = false;
        let mut tempo_events = Vec::new();

        let mut time_in_units = 0.0;
        for event in track.iter() {
            // time_in_units += event.delta.as_int() as f64;
            match &event.kind {
                EventKind::Midi {
                    channel: _,
                    message: _,
                } => {
                    time_in_units += event.delta.as_int() as f64;
                }
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
                    MetaMessage::TimeSignature(_data0, _data1, _, _) => {
                        //time_signature =data0.to_owned() as f64 / u32::pow(2, data1.to_owned() as u32) as f64;
                    }
                    _ => {}
                },
                _ => {}
            };
        }

        // let u_per_quarter_note = u_per_quarter_note as f64;
        // let u_time = tempo as f64 / u_per_quarter_note;
        // let has_tempo = tempo != 500000;

        // let seconds_per_measure = MidiTrack::measure_duration(tempo, time_signature);

        MidiTrack {
            tempo,
            tempo_events,
            // u_per_quarter_note,
            // u_time,
            // time_signature,
            has_tempo,
            track_id,
            // seconds_per_measure,
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
        for event in events.iter() {
            use midly::EventKind;
            use midly::MidiMessage;

            match &event.kind {
                EventKind::Midi { channel, message } => {
                    time_in_units += event.delta.as_int() as f64;

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
                        MidiMessage::ProgramChange(_data0) => {
                            // TODO Find Out Why Ignoring This Event Gives Better Resoults
                            time_in_units -= event.delta.as_int() as f64;
                            // println!("{} ProgramChange", index);
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


        /*


        let mut time_in_units = 0.0;

        struct Note {
            time_in_units: f64,
            vel: u8,
            channel: u8,
        };
        let mut current_notes: HashMap<u8, Note> = HashMap::new();
        // println!("NEW TRACK");


        let mut index = 0;
        for event in events.iter() {
            use midly::EventKind;
            use midly::MetaMessage;
            use midly::MidiMessage;

            match &event.kind {
                EventKind::Midi { channel, message } => {
                    time_in_units += event.delta.as_int() as f64;
                    // println!("time_in_units:{}", time_in_units);
                    match &message {
                        MidiMessage::NoteOn(data0, data1) => {
                            let data0 = data0.as_int();
                            let data1 = data1.as_int();
                            if data1 > 0 && data0 >= lower_bound && data0 <= higher_bound {
                                let k = data0 - lower_bound;
                                current_notes.insert(
                                    k,
                                    Note {
                                        time_in_units: time_in_units,
                                        vel: data1,
                                        channel: channel.as_int(),
                                    },
                                );
                            // println!("{:?}", current_notes.get(&k).unwrap());
                            } else if data1 == 0 {
                                let k = data0 - lower_bound;

                                if current_notes.contains_key(&k) {
                                    let n = current_notes.get(&k).unwrap();

                                    let start = self.u_time * n.time_in_units;
                                    let duration = self.u_time * time_in_units - start;

                                    let mn = MidiNote {
                                        start: start / 1000000.0,
                                        duration: duration / 1000000.0,
                                        note: k,
                                        vel: n.vel,
                                        ch: channel.as_int(),
                                        track_id: 0, // Placeholder
                                    };

                                    // let bpm = 60_000_000 / self.tempo as u64;
                                    // println!("N:{} BPM {}", mn.note, bpm);
                                    // println!("{:?}", mn);
                                    self.notes.push(mn);
                                }
                            }

                        }
                        MidiMessage::NoteOff(data0, data1) => {
                            let data0 = data0.as_int();

                            let k = data0 - lower_bound;

                            if current_notes.contains_key(&k) {
                                let n = current_notes.get(&k).unwrap();

                                let start = self.u_time * n.time_in_units;
                                let duration = self.u_time * time_in_units - start;

                                let mn = MidiNote {
                                    start: start / 1000000.0,
                                    duration: duration / 1000000.0,
                                    note: k,
                                    vel: n.vel,
                                    ch: channel.as_int(),
                                    track_id: 0, // Placeholder
                                };

                                // let bpm = 60_000_000 / self.tempo as u64;
                                // println!("N:{} BPM {}", mn.note, bpm);
                                // println!("{:?}", mn);
                                self.notes.push(mn);
                            }

                        }
                        MidiMessage::ProgramChange(_data0) => {
                            time_in_units -= event.delta.as_int() as f64;
                            // println!("{} ProgramChange", index);
                        }
                        _ => {}
                    }
                }
                EventKind::Meta(meta) => match &meta {
                    MetaMessage::Tempo(t) => {
                        self.tempo = t.as_int();
                        self.u_time = self.tempo as f64 / self.u_per_quarter_note;

                        self.seconds_per_measure =
                            MidiTrack::measure_duration(self.tempo, self.time_signature);

                        let bpm = 60_000_000 / self.tempo as u64;
                        println!("BPM {}", bpm);
                    }
                    MetaMessage::TimeSignature(data0, data1, _, _) => {
                        self.seconds_per_measure /= self.time_signature;
                        self.time_signature =
                            data0.to_owned() as f64 / u32::pow(2, data1.to_owned() as u32) as f64;
                        self.seconds_per_measure *= self.time_signature;
                    }
                    _ => {}
                },
                _ => {}
            }

        }

        self.notes
            .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());

        */
    }
}
