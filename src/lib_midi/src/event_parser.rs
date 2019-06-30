
use crate::tracks_parser::TracksParser;
use midly::EventKind;
use midly::MetaMessage;
use midly::MidiMessage;

use std::collections::HashMap;


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

pub struct TempNote {
    time_in_units: f64,
    vel: u8,
    channel: u8,
}

pub struct EventParser {
    time_in_units: f64,
    time_offset: f64,
    note_map: HashMap<u8, TempNote>,
    pub track_id: usize,
    pub notes: Vec<MidiNote>,
}

impl EventParser {
    pub fn new(i: usize) -> EventParser {
        EventParser {
            time_in_units: 0.0,
            time_offset: 0.0,
            note_map: HashMap::new(),
            track_id: i,
            notes: Vec::new(),
        }
    }
    pub fn parse_event(
        &mut self,
        event: &midly::Event,
        id: usize,
        parent_parser: &mut TracksParser,
    ) {
        match &event.kind {
            EventKind::Midi { channel, message } => {
                self.time_in_units += event.delta.as_int() as f64;
                // println!("time_in_units:{}", time_in_units);
                match &message {
                    MidiMessage::NoteOn(data0, data1) => {
                        let data0 = data0.as_int();
                        let data1 = data1.as_int();
                        if data1 > 0
                        /*&& data0 >= lower_bound && data0 <= higher_bound*/
                        {
                            let k = data0 /*- lower_bound*/;
                            self.note_map.insert(
                                k,
                                TempNote {
                                    time_in_units: self.time_in_units,
                                    vel: data1,
                                    channel: channel.as_int(),
                                },
                            );

                        } else if data1 == 0 {
                            let k = data0 /*- lower_bound*/;

                            if self.note_map.contains_key(&k) {
                                let n = self.note_map.get(&k).unwrap();

                                // let start = (parent_parser.u_time * n.time_in_units);
                                // let duration = parent_parser.u_time * self.time_in_units - start;


                                let start = parent_parser.pulses_to_ms(n.time_in_units) / 1000.0;
                                let duration =
                                    parent_parser.pulses_to_ms(self.time_in_units) / 1000.0 - start;

                                let mn = MidiNote {
                                    start: start,
                                    duration: duration,
                                    note: k,
                                    vel: n.vel,
                                    ch: channel.as_int(),
                                    track_id: self.track_id,
                                    id: id,
                                };

                                let bpm = 60_000_000 / parent_parser.tempo as u64;

                                // println!("{:?}", mn);
                                println!(
                                    "T:{} ID:{} N:{} S:{:.1} \t B:{} \t Offset:{} ",
                                    self.track_id, id, k, mn.start, bpm, self.time_offset
                                );
                                self.notes.push(mn);


                                // let bpm = 60_000_000 / parent_parser.tempo as u64;
                                // println!("I:{} \t B:{}", self.track_id, bpm);
                                // println!("N:{} BPM {}", mn.note, bpm);
                                // self.notes.push(mn);
                            }
                        }

                    }
                    MidiMessage::NoteOff(data0, data1) => {
                        let k = data0.as_int() /*- lower_bound*/;

                        if self.note_map.contains_key(&k) {
                            let n = self.note_map.get(&k).unwrap();

                            // let start = (parent_parser.u_time * n.time_in_units);
                            // let duration = parent_parser.u_time * self.time_in_units - start;


                            let start = parent_parser.pulses_to_ms(n.time_in_units) / 1000.0;
                            let duration =
                                parent_parser.pulses_to_ms(self.time_in_units) / 1000.0 - start;


                            let mn = MidiNote {
                                start: start,
                                duration: duration,
                                note: k,
                                vel: n.vel,
                                ch: channel.as_int(),
                                track_id: self.track_id,
                                id: id,
                            };

                            let bpm = 60_000_000 / parent_parser.tempo as u64;

                            // println!("{:?}", mn);
                            println!(
                                "T:{} ID:{} N:{} S:{:.1} \t B:{} \t Offset:{} ",
                                self.track_id, id, k, mn.start, bpm, self.time_offset
                            );
                            self.notes.push(mn);

                            // let bpm = 60_000_000 / parent_parser.tempo as u64;
                            // println!("I:{} \t B:{}", self.track_id, bpm);
                            // println!("N:{} BPM {}", mn.note, bpm);
                            // self.notes.push(mn);
                        }
                    }
                    MidiMessage::ProgramChange(_data0) => {
                        // self.time_in_units -= event.delta.as_int() as f64;
                        // println!("{} ProgramChange", index);
                    }
                    _ => {}
                }
            }
            EventKind::Meta(meta) => {
                match &meta {
                    MetaMessage::Tempo(t) => {
                        println!("TEMPO Track {}", self.track_id);

                        // let delta_pulses = 0;
                        // if event.delta
                        // self.time_in_units = -= event.delta;
                        // self.time_offset = parent_parser.u_time * self.time_in_units / 1000000.0;

                        // println!("TO:{}", self.time_offset);
                        // ====
                        // FIXME: Timeing is broken after change of tempo
                        // parent_parser.update_timing(t.as_int());
                        // ====
                        // parent_parser.tempo = t.as_int();
                        // parent_parser.u_time = parent_parser.tempo as f64 / parent_parser.u_per_quarter_note;

                        // self.seconds_per_measure =
                        //     MidiTrack::measure_duration(self.tempo, self.time_signature);

                        // let bpm = 60_000_000 / t.as_int() as u64;
                        // println!("New BPM {}", bpm);
                    }
                    MetaMessage::TimeSignature(data0, data1, _, _) => {
                        // self.seconds_per_measure /= self.time_signature;
                        // self.time_signature =
                        //     data0.to_owned() as f64 / u32::pow(2, data1.to_owned() as u32) as f64;
                        // self.seconds_per_measure *= self.time_signature;
                    }
                    _ => {}
                }

            }

            _ => {}
        }
    }
}

