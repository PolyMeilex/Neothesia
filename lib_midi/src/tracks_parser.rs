use std::time::Duration;

use {
    crate::{MidiTrack, TempoEvent},
    midly::TrackEvent,
};

pub struct TracksParser {
    tempo_events: Vec<TempoEvent>,
    u_per_quarter_note: u16,
}

impl TracksParser {
    pub fn new(u_per_quarter_note: u16) -> Self {
        Self {
            tempo_events: Vec::new(),
            u_per_quarter_note,
        }
    }

    pub fn parse(&mut self, tracks: &mut Vec<MidiTrack>, midly_tracks: &[Vec<TrackEvent>]) {
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
            // TODO: Return to caller to inform user that fallback bpm is used
            println!("There is no tempo track! Useing 120 bpm as fallback");

            //panic!("There is no track with tempo info"); // ! For Debug Only
        }

        for trk in tracks.iter_mut() {
            trk.extract_notes(&midly_tracks[trk.track_id], self);
        }
    }

    fn p_to_micro(&self, delta_pulses: u64, tempo: u32) -> Duration {
        let u_time = delta_pulses as f64 / self.u_per_quarter_note as f64;
        // We floor only because Synthesia floors,
        // so if we want to test for timing regresions we have to do the same
        let time = (u_time * tempo as f64).floor() as u64;
        Duration::from_micros(time)
    }

    pub fn pulses_to_micro(&self, event_pulses: u64) -> Duration {
        let mut res = Duration::ZERO;

        let mut hit = false;
        let mut last_tempo_event_pulses = 0u64;
        let mut running_tempo = 500_000;

        let event_pulses = event_pulses;

        for tempo_event in self.tempo_events.iter() {
            let tempo_event_pulses = tempo_event.time_in_units;

            let delta_pulses = if event_pulses > tempo_event_pulses {
                tempo_event_pulses - last_tempo_event_pulses
            } else {
                hit = true;
                event_pulses - last_tempo_event_pulses
            };

            res += self.p_to_micro(delta_pulses, running_tempo);

            if hit {
                break;
            }

            running_tempo = tempo_event.tempo;
            last_tempo_event_pulses = tempo_event_pulses;
        }

        if !hit {
            let remaining_pulses = event_pulses - last_tempo_event_pulses;
            res += self.p_to_micro(remaining_pulses, running_tempo);
        }

        res
    }
}
