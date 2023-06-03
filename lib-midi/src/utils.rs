use crate::TempoEvent;
use midly::{MetaMessage, TrackEvent, TrackEventKind};
use std::{collections::HashMap, time::Duration};

fn pulse_to_duration(pulses: u64, tempo: u32, pulses_per_quarter_note: u16) -> Duration {
    let u_time = pulses as f64 / pulses_per_quarter_note as f64;
    // We floor only because Synthesia floors,
    // so if we want to test for timing regresions we have to do the same
    let time = (u_time * tempo as f64).floor() as u64;
    Duration::from_micros(time)
}

pub fn pulses_to_duration(
    tempo_events: &[TempoEvent],
    event_pulses: u64,
    pulses_per_quarter_note: u16,
) -> Duration {
    let mut res = Duration::ZERO;

    let mut hit = false;
    let mut last_tempo_event_pulses = 0u64;
    let mut running_tempo = 500_000;

    let event_pulses = event_pulses;

    for tempo_event in tempo_events.iter() {
        let tempo_event_pulses = tempo_event.absolute_pulses;

        // If the time we're asking to convert is still beyond
        // this tempo event, just add the last time slice (at
        // the previous tempo) to the running wall-clock time.
        let delta_pulses = if event_pulses > tempo_event_pulses {
            tempo_event_pulses - last_tempo_event_pulses
        } else {
            hit = true;
            event_pulses - last_tempo_event_pulses
        };

        res += pulse_to_duration(delta_pulses, running_tempo, pulses_per_quarter_note);

        // If the time we're calculating is before the tempo event we're
        // looking at, we're done.
        if hit {
            break;
        }

        running_tempo = tempo_event.tempo;
        last_tempo_event_pulses = tempo_event_pulses;
    }

    if !hit {
        let remaining_pulses = event_pulses - last_tempo_event_pulses;
        res += pulse_to_duration(remaining_pulses, running_tempo, pulses_per_quarter_note);
    }

    res
}

pub struct TempoTrack(Vec<TempoEvent>);

impl std::ops::Deref for TempoTrack {
    type Target = Vec<TempoEvent>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TempoTrack {
    pub fn build(track_events: &[Vec<TrackEvent>]) -> TempoTrack {
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
                            tempo: t.as_int(),
                        },
                    );
                };
            }
        }

        let mut tempo_events: Vec<_> = tempo_events.into_values().collect();
        tempo_events.sort_by_key(|e| e.absolute_pulses);

        let mut previous_absolute_pulses = 0u64;
        let tempo_events: Vec<_> = tempo_events
            .into_iter()
            .map(|mut e| {
                let absolute_pulses = e.absolute_pulses;

                let relative = absolute_pulses - previous_absolute_pulses;
                previous_absolute_pulses = absolute_pulses;

                e.relative_pulses = relative;

                e
            })
            .collect();

        TempoTrack(tempo_events)
    }
}
