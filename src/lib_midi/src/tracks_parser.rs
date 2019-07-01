use crate::track::MidiTrack;

pub struct TracksParser {
    tempo_events: Vec<crate::track::TempoEvent>,
    u_per_quarter_note: f64,
}

impl TracksParser {
    pub fn new(u_per_quarter_note: u16) -> TracksParser {
        let u_per_quarter_note = u_per_quarter_note as f64;

        TracksParser {
            tempo_events: Vec::new(),
            u_per_quarter_note,
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

        // TODO: Merge tempo events if there is more than one tempo track
        if tracks[tempo_track].has_tempo {
            self.tempo_events = tracks[tempo_track].tempo_events.clone();
        } else {
            println!("There is no tempo track! Useing 120 bpm as fallback");

            panic!("There is no track with tempo info"); // ! For Debug Only
        }

        for trk in tracks.iter_mut() {
            trk.extract_notes(&midly_tracks[trk.track_id], self);
        }

    }
    fn p_to_ms(&self, time_in_units: f64, tempo: u32) -> f64 {
        let u_time = tempo as f64 / self.u_per_quarter_note;
        return u_time * time_in_units / 1000.0;
    }
    pub fn pulses_to_ms(&self, event_pulses: f64) -> f64 {
        let mut res: f64 = 0.0;

        let mut hit = false;
        let mut last_tempo_event_pulses: f64 = 0.0;
        let mut running_tempo = 500000;

        let event_pulses = event_pulses;

        for tempo_event in self.tempo_events.iter() {
            let tempo_event_pulses = tempo_event.time_in_units;

            let delta_pulses: f64;
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

