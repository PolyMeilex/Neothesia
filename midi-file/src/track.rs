use midly::{MidiMessage, TrackEvent, TrackEventKind, num::u4};
use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::tempo_track::TempoTrack;

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub channel: u8,
    pub timestamp: Duration,
    pub message: MidiMessage,
    pub track_id: usize,
    pub track_color_id: usize,
}

#[derive(Debug, Clone)]
pub struct ProgramEvent {
    pub channel: u8,
    pub timestamp: Duration,
    pub program: u8,
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
    pub track_color_id: usize,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
    // Translated notes with calculated timings
    pub notes: Arc<[MidiNote]>,

    pub events: Arc<[MidiEvent]>,

    pub track_id: usize,
    pub track_color_id: usize,

    pub programs: Arc<[ProgramEvent]>,
    pub has_drums: bool,
    pub has_other_than_drums: bool,
}

impl MidiTrack {
    pub fn new(
        track_id: usize,
        track_color_id: usize,
        tempo_track: &TempoTrack,
        track_events: &[TrackEvent],
    ) -> Self {
        let (
            events,
            EventsBuilder {
                programs,
                notes,
                has_drums,
                has_other_than_drums,
                ..
            },
        ) = build(track_id, track_color_id, tempo_track, track_events);

        Self {
            track_id,
            track_color_id,
            notes: notes.into(),
            events: events.into(),
            programs: programs.into(),
            has_drums,
            has_other_than_drums,
        }
    }
}

struct NoteInfo {
    velocity: u8,
    channel: u8,
    timestamp: Duration,
}

#[derive(Default)]
struct EventsBuilder {
    programs: Vec<ProgramEvent>,
    has_drums: bool,
    has_other_than_drums: bool,

    active_notes: HashMap<u8, NoteInfo>,
    notes: Vec<MidiNote>,
}

impl EventsBuilder {
    fn build_notes(
        &mut self,
        channel: u4,
        message: &MidiMessage,
        timestamp: Duration,
        track_id: usize,
        track_color_id: usize,
    ) {
        let (key, velocity) = match message {
            MidiMessage::NoteOn { vel, key } => (key.as_int(), vel.as_int()),
            MidiMessage::NoteOff { vel, key } => (key.as_int(), vel.as_int()),
            _ => {
                return;
            }
        };

        if let Some(active) = self.active_notes.remove(&key) {
            let start = active.timestamp;
            let end = timestamp;
            let duration = end - start;

            let note = MidiNote {
                start,
                end,
                duration,
                note: key,
                velocity: active.velocity,
                channel: active.channel,
                track_id,
                track_color_id,
            };

            self.notes.push(note);
        }

        if let MidiMessage::NoteOn { .. } = message {
            let note = NoteInfo {
                channel: channel.as_int(),
                velocity,
                timestamp,
            };
            self.active_notes.insert(key, note);
        }
    }

    fn check_for_drums(&mut self, channel: u4) {
        if channel == 9 || channel == 15 {
            self.has_drums = true;
        } else {
            self.has_other_than_drums = true;
        }
    }

    fn on_event(
        &mut self,
        channel: u4,
        message: MidiMessage,
        timestamp: Duration,
        track_id: usize,
        track_color_id: usize,
    ) -> MidiEvent {
        let message = match message {
            midly::MidiMessage::NoteOn { key, vel } => {
                self.check_for_drums(channel);

                if vel.as_int() > 0 {
                    message
                } else {
                    midly::MidiMessage::NoteOff { key, vel }
                }
            }
            midly::MidiMessage::ProgramChange { program } => {
                self.programs.push(ProgramEvent {
                    timestamp,
                    channel: channel.as_int(),
                    program: program.as_int(),
                });
                message
            }
            message => message,
        };

        self.build_notes(channel, &message, timestamp, track_id, track_color_id);

        MidiEvent {
            channel: channel.as_int(),
            timestamp,
            message,
            track_id,
            track_color_id,
        }
    }
}

fn build(
    track_id: usize,
    track_color_id: usize,
    tempo_track: &TempoTrack,
    track_events: &[TrackEvent],
) -> (Vec<MidiEvent>, EventsBuilder) {
    let mut builder = EventsBuilder::default();

    let mut pulses: u64 = 0;
    let events = track_events
        .iter()
        .filter_map(|event| {
            pulses += event.delta.as_int() as u64;
            match event.kind {
                TrackEventKind::Midi { channel, message } => {
                    let timestamp = tempo_track.pulses_to_duration(pulses);
                    Some(builder.on_event(channel, message, timestamp, track_id, track_color_id))
                }
                _ => None,
            }
        })
        .collect();

    (events, builder)
}
