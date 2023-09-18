use crate::TempoEvent;
use midly::{MetaMessage, TrackEvent, TrackEventKind};
use std::{collections::HashMap, time::Duration};

pub struct TempoTrack {
    pulses_per_quarter_note: u16,
    events: Vec<TempoEvent>,
}

impl std::ops::Deref for TempoTrack {
    type Target = Vec<TempoEvent>;

    fn deref(&self) -> &Self::Target {
        &self.events
    }
}

impl TempoTrack {
    pub fn build(track_events: &[Vec<TrackEvent>], pulses_per_quarter_note: u16) -> TempoTrack {
        // This map will help us get rid of duplicate events if
        // the tempo is specified in every track (as is common).
        let mut tempo_events: HashMap<u64, TempoEvent> = HashMap::new();

        for events in track_events.iter() {
            let mut pulses: u64 = 0;
            for event in events.iter() {
                pulses += event.delta.as_int() as u64;

                if let TrackEventKind::Meta(MetaMessage::Tempo(t)) = &event.kind {
                    tempo_events.insert(
                        pulses,
                        TempoEvent {
                            absolute_pulses: pulses,
                            relative_pulses: 0,
                            timestamp: Duration::ZERO,
                            tempo: t.as_int(),
                        },
                    );
                };
            }
        }

        let mut tempo_events: Vec<_> = tempo_events.into_values().collect();
        tempo_events.sort_by_key(|e| e.absolute_pulses);

        let mut previous_absolute_pulses = 0u64;
        let mut running_tempo = 500_000;
        let mut res = Duration::ZERO;

        for tempo_event in tempo_events.iter_mut() {
            let tempo_event_pulses = tempo_event.absolute_pulses;

            tempo_event.relative_pulses = tempo_event_pulses - previous_absolute_pulses;

            res += pulse_to_duration(
                tempo_event.relative_pulses,
                running_tempo,
                pulses_per_quarter_note,
            );

            tempo_event.timestamp = res;

            running_tempo = tempo_event.tempo;
            previous_absolute_pulses = tempo_event_pulses;
        }

        TempoTrack {
            pulses_per_quarter_note,
            events: tempo_events,
        }
    }

    pub fn pulses_to_duration(&self, event_pulses: u64) -> Duration {
        let mut res = Duration::ZERO;

        let mut hit = false;
        let mut previous_absolute_pulses = 0u64;
        let mut running_tempo = 500_000;

        let event_pulses = event_pulses;

        let id = match self
            .events
            .binary_search_by_key(&event_pulses, |e| e.absolute_pulses)
        {
            Ok(id) => id.saturating_sub(1),
            Err(id) => id.saturating_sub(1),
        };

        for tempo_event in self.events.iter().skip(id) {
            let tempo_event_pulses = tempo_event.absolute_pulses;

            // If the time we're asking to convert is still beyond
            // this tempo event, just add the last time slice (at
            // the previous tempo) to the running wall-clock time.
            if event_pulses > tempo_event_pulses {
                res = tempo_event.timestamp;

                running_tempo = tempo_event.tempo;
                previous_absolute_pulses = tempo_event_pulses;
            } else {
                hit = true;
                let delta_pulses = event_pulses - previous_absolute_pulses;
                res += pulse_to_duration(delta_pulses, running_tempo, self.pulses_per_quarter_note);

                // If the time we're calculating is before the tempo event we're
                // looking at, we're done.
                break;
            }
        }

        if !hit {
            let remaining_pulses = event_pulses - previous_absolute_pulses;
            res += pulse_to_duration(
                remaining_pulses,
                running_tempo,
                self.pulses_per_quarter_note,
            );
        }

        res
    }
}

fn pulse_to_duration(pulses: u64, tempo: u32, pulses_per_quarter_note: u16) -> Duration {
    let u_time = pulses as f64 / pulses_per_quarter_note as f64;
    // We floor only because Synthesia floors,
    // so if we want to test for timing regresions we have to do the same
    let time = (u_time * tempo as f64).floor() as u64;
    Duration::from_micros(time)
}
