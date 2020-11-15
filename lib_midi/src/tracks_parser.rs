use {
    crate::{MidiTrack, TempoEvent},
    midly::TrackEvent,
};

pub struct TracksParser {
    tempo_events: Vec<TempoEvent>,
    u_per_quarter_note: f32,
}

impl TracksParser {
    pub fn new(u_per_quarter_note: u16) -> Self {
        let u_per_quarter_note = f32::from(u_per_quarter_note);

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
    fn p_to_ms(&self, time_in_units: f32, tempo: u32) -> f32 {
        let u_time = tempo as f32 / self.u_per_quarter_note;
        u_time * time_in_units / 1000.0
    }
    pub fn pulses_to_ms(&self, event_pulses: f32) -> f32 {
        let mut res: f32 = 0.0;

        let mut hit = false;
        let mut last_tempo_event_pulses: f32 = 0.0;
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

        res
    }
}
