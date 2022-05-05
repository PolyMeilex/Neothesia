use std::time::Duration;

use crate::{MidiEvent, MidiTrack};

#[derive(Debug, Clone)]
pub struct PlaybackState {
    running: Duration,
    leed_in: Duration,
    seen_events: usize,

    first_note_start: Duration,
    last_note_end: Duration,
}

impl PlaybackState {
    pub fn new(leed_in: Duration, track: &MidiTrack) -> Self {
        let first_note_start = if let Some(note) = track.notes.first() {
            note.start
        } else {
            Duration::ZERO
        };
        let last_note_end = if let Some(note) = track.notes.last() {
            note.start + note.duration
        } else {
            Duration::ZERO
        };

        Self {
            running: Duration::ZERO,
            leed_in,
            seen_events: 0,

            first_note_start,
            last_note_end,
        }
    }

    pub fn update(&mut self, track: &MidiTrack, delta: Duration) -> Vec<MidiEvent> {
        self.running += delta;

        track
            .events
            .iter()
            .skip(self.seen_events)
            .filter(|event| event.timestamp + self.leed_in <= self.running)
            .map(|event| {
                let event = event.clone();
                self.seen_events += 1;
                event
            })
            .collect()
    }

    pub fn time(&self) -> Duration {
        self.running
    }

    pub fn set_time(&mut self, time: Duration) {
        self.reset();
        self.running = time;
    }

    pub fn percentage(&self) -> f32 {
        self.running.as_secs_f32() / self.last_note_end.as_secs_f32()
    }

    pub fn first_note_start(&self) -> &Duration {
        &self.first_note_start
    }

    pub fn last_note_end(&self) -> &Duration {
        &self.last_note_end
    }

    pub fn reset(&mut self) {
        self.running = Duration::ZERO;
        self.seen_events = 0;
    }
}
