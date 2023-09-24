use std::time::Duration;

use crate::{MergedTracks, MidiEvent};

#[derive(Debug, Clone)]
pub struct PlaybackState {
    tracks: MergedTracks,
    is_paused: bool,
    running: Duration,
    leed_in: Duration,
    seen_events: usize,

    first_note_start: Duration,
    last_note_end: Duration,
}

impl PlaybackState {
    pub fn new(leed_in: Duration, tracks: MergedTracks) -> Self {
        let first_note_start = if let Some(note) = tracks.notes.first() {
            note.start
        } else {
            Duration::ZERO
        };
        let last_note_end = if let Some(note) = tracks.notes.last() {
            note.start + note.duration
        } else {
            Duration::ZERO
        };

        Self {
            tracks,
            is_paused: false,
            running: Duration::ZERO,
            leed_in,
            seen_events: 0,

            first_note_start,
            last_note_end,
        }
    }

    pub fn update(&mut self, delta: Duration) -> Vec<&MidiEvent> {
        if !self.is_paused {
            self.running += delta;
        }

        let events: Vec<_> = self.tracks.events[self.seen_events..]
            .iter()
            .take_while(|event| event.timestamp + self.leed_in <= self.running)
            .collect();

        self.seen_events += events.len();
        events
    }

    pub fn is_paused(&self) -> bool {
        self.is_paused
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    pub fn time(&self) -> Duration {
        self.running
    }

    pub fn set_time(&mut self, time: Duration) {
        self.reset();
        self.running = time;
    }

    pub fn percentage(&self) -> f32 {
        self.running.as_secs_f32() / self.lenght().as_secs_f32()
    }

    pub fn leed_in(&self) -> &Duration {
        &self.leed_in
    }

    pub fn first_note_start(&self) -> &Duration {
        &self.first_note_start
    }

    pub fn last_note_end(&self) -> &Duration {
        &self.last_note_end
    }

    pub fn lenght(&self) -> Duration {
        self.last_note_end + self.leed_in
    }

    pub fn reset(&mut self) {
        self.running = Duration::ZERO;
        self.seen_events = 0;
    }
}
