
use crate::event_parser::EventParser;
use crate::track::MidiTrack;

use std::collections::HashMap;

pub struct TracksParser {
    pub tempo: u32,
    tempo_events: Vec<crate::track::TempoEvent>,
    u_per_quarter_note: f64,
    pub u_time: f64,
    //time_signature: f64,
    //seconds_per_measure: f64,
}

impl TracksParser {
    pub fn new(u_per_quarter_note: u16) -> TracksParser {
        let tempo = 500000; // 120 bpm
        let u_per_quarter_note = u_per_quarter_note as f64;
        let u_time = tempo as f64 / u_per_quarter_note;

        //let time_signature = 0.0;
        //let seconds_per_measure = 0.0;

        TracksParser {
            tempo,
            tempo_events: Vec::new(),
            u_per_quarter_note,
            u_time,
            //time_signature,
            // seconds_per_measure,
        }
    }
    pub fn parse(&mut self, tracks: &mut Vec<MidiTrack>, midly_tracks: &Vec<Vec<midly::Event>>) {
        let mut tempo_track = 0;
        for (i, trk) in tracks.iter().enumerate() {
            if trk.has_tempo {
                tempo_track = i;
                break;
            }
        }

        if tracks[tempo_track].has_tempo {
            self.tempo_events = tracks[tempo_track].tempo_events.clone();
        // self.update_timing(60_000_000 / 130);
        // self.update_timing(tracks[tempo_track].tempo);
        // self.tempo = tracks[tempo_track].tempo;
        } else {
            println!("There is no tempo track! Useing 120 bpm as fallback");

            panic!("There is no track with tempo info"); // For Debug Only
        }

        tracks.sort_by(|a, b| a.has_tempo.cmp(&b.has_tempo).reverse());

        // println!("rt0:{:?}", tracks);

        let mut event_parsers: Vec<EventParser> = Vec::new();

        for (i, _) in tracks.iter().enumerate() {
            event_parsers.push(EventParser::new(i));
        }

        // TODO: MOVE EVENT PARSER BACK from event_parser.rs TO track.rs
        for ep in event_parsers.iter_mut() {
            for (i, event) in midly_tracks[ep.track_id].iter().enumerate() {
                ep.parse_event(&event, i, self);
            }
        }

        for (i, trk) in tracks.iter_mut().enumerate() {
            event_parsers[i]
                .notes
                .sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap());
            trk.notes = event_parsers[i].notes.clone();
        }

    }
    pub fn update_timing(&mut self, tempo: u32) {
        // if 60_000_000 / tempo == 70 {
        self.tempo = tempo;
        self.u_time = tempo as f64 / self.u_per_quarter_note;
        // u_time * time_in_units

        // }
        // println!("NEW TEMPO {}", 60_000_000 / tempo);
    }
    pub fn p_to_ms(&self, time_in_units: f64, tempo: u32) -> f64 {
        let u_time = tempo as f64 / self.u_per_quarter_note;
        return u_time * time_in_units / 1000000.0 * 1000.0;
    }
    pub fn pulses_to_ms(&self, event_pulses: f64) -> f64 {
        let mut res: f64 = 0.0;

        let mut hit = false;
        let mut last_tempo_event_pulses: f64 = 0.0;
        let mut running_tempo = 500000;

        let event_pulses = event_pulses;

        for tempo_event in self.tempo_events.iter() {
            let tempo_event_pulses = tempo_event.time_in_units;

            let mut delta_pulses: f64 = 0.0;
            if event_pulses > tempo_event_pulses {
                delta_pulses = tempo_event_pulses - last_tempo_event_pulses;
            } else {
                hit = true;
                delta_pulses = event_pulses - last_tempo_event_pulses;
            }

            res += self.p_to_ms(delta_pulses, running_tempo);

            if hit {
                break;
            }

            running_tempo = tempo_event.tempo;
            last_tempo_event_pulses = tempo_event_pulses;
        }

        if !hit {
            let remaining_pulses = event_pulses - last_tempo_event_pulses;
            res += self.p_to_ms(remaining_pulses, running_tempo);
        }

        return res;
    }
}

