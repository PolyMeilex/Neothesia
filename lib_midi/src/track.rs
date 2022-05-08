use std::time::Duration;

use crate::{pulses_to_duration, TempoTrack};

use {
    midly::{MidiMessage, TrackEvent, TrackEventKind},
    std::collections::HashMap,
};

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub channel: u8,
    pub delta: u32,
    pub timestamp: Duration,
    pub message: MidiMessage,
    pub track_id: usize,
}

#[derive(Debug, Clone)]
pub struct TempoEvent {
    pub absolute_pulses: u64,
    pub relative_pulses: u64,
    /// Tempo in microseconds per quarter note.
    pub tempo: u32,
}

#[derive(Debug, Clone)]
pub struct MidiNote {
    pub start: Duration,
    pub end: Duration,
    pub duration: Duration,
    pub note: u8,
    pub velocity: u8,
    pub channel: u8,
    pub track_id: usize,
    pub id: usize,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
    // Translated notes with calculated timings
    pub notes: Vec<MidiNote>,

    pub events: Vec<MidiEvent>,

    pub track_id: usize,
}

impl MidiTrack {
    pub fn new(
        track_id: usize,
        tempo_events: &TempoTrack,
        track_events: &[TrackEvent],
        pulses_per_quarter_note: u16,
    ) -> Self {
        let notes = build_notes(
            track_id,
            tempo_events,
            track_events,
            pulses_per_quarter_note,
        );

        let mut pulses: u64 = 0;
        let events = track_events
            .iter()
            .filter_map(|event| {
                pulses += event.delta.as_int() as u64;
                match event.kind {
                    TrackEventKind::Midi { channel, message } => {
                        let message = match message {
                            midly::MidiMessage::NoteOn { key, vel } => {
                                if vel.as_int() > 0 {
                                    message
                                } else {
                                    midly::MidiMessage::NoteOff { key, vel }
                                }
                            }
                            message => message,
                        };

                        Some(MidiEvent {
                            channel: channel.as_int(),
                            delta: event.delta.as_int(),
                            timestamp: pulses_to_duration(
                                tempo_events,
                                pulses,
                                pulses_per_quarter_note,
                            ),
                            message,
                            track_id,
                        })
                    }
                    _ => None,
                }
            })
            .collect();

        Self {
            track_id,
            notes,
            events,
        }
    }
}

fn build_notes(
    track_id: usize,
    tempo_events: &TempoTrack,
    track_events: &[TrackEvent],
    pulses_per_quarter_note: u16,
) -> Vec<MidiNote> {
    struct NoteInfo {
        velocity: u8,
        channel: u8,
        pulses: u64,
    }

    let mut active_notes: HashMap<u8, NoteInfo> = HashMap::new();
    let mut notes = Vec::new();

    let mut pulses: u64 = 0;
    for event in track_events.iter() {
        pulses += event.delta.as_int() as u64;

        if let TrackEventKind::Midi { channel, message } = event.kind {
            let (key, velocity) = match message {
                MidiMessage::NoteOn { vel, key } => (key.as_int(), vel.as_int()),
                MidiMessage::NoteOff { vel, key } => (key.as_int(), vel.as_int()),
                _ => {
                    continue;
                }
            };

            if let Some(active) = active_notes.remove(&key) {
                let start = active.pulses;
                let end = pulses;

                let start = pulses_to_duration(tempo_events, start, pulses_per_quarter_note);
                let end = pulses_to_duration(tempo_events, end, pulses_per_quarter_note);
                let duration = end - start;

                let note = MidiNote {
                    start,
                    end,
                    duration,
                    note: key,
                    velocity: active.velocity,
                    channel: active.channel,
                    track_id,
                    id: notes.len(),
                };

                notes.push(note);
            }

            let on = matches!(&message, MidiMessage::NoteOn { .. }) && velocity > 0;

            if on {
                let note = NoteInfo {
                    channel: channel.as_int(),
                    velocity,
                    pulses,
                };
                active_notes.insert(key, note);
            }
        }
    }

    notes
}
