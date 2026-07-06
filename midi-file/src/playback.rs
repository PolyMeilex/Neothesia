use std::{sync::Arc, time::Duration};

use crate::{MidiEvent, MidiTrack};

#[derive(Debug, Clone)]
struct TrackState {
    seen_events: usize,
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    tracks: Arc<[MidiTrack]>,
    track_states: Box<[TrackState]>,
    is_paused: bool,
    running: Duration,
    leed_in: Duration,

    first_note_start: Duration,
    last_note_end: Duration,
}

impl PlaybackState {
    pub fn new(leed_in: Duration, tracks: Arc<[MidiTrack]>) -> Self {
        let mut first_note_start = Duration::ZERO;
        let mut last_note_end = Duration::ZERO;

        for track in tracks.iter() {
            if let Some(note) = track.notes.first() {
                first_note_start = first_note_start.min(note.start);
            }

            if let Some(note) = track.notes.last() {
                last_note_end = last_note_end.max(note.start + note.duration);
            }
        }

        let track_states = vec![TrackState { seen_events: 0 }; tracks.len()];

        Self {
            tracks,
            track_states: track_states.into(),
            is_paused: false,
            running: Duration::ZERO,
            leed_in,

            first_note_start,
            last_note_end,
        }
    }

    pub fn update(&mut self, delta: Duration) -> Vec<&MidiEvent> {
        if !self.is_paused {
            self.running += delta;
        }

        let events: Vec<_> = self
            .tracks
            .iter()
            .zip(self.track_states.iter_mut())
            .flat_map(|(track, state)| {
                track.events[state.seen_events..]
                    .iter()
                    .take_while(|event| event.timestamp + self.leed_in <= self.running)
                    .inspect(|_| state.seen_events += 1)
            })
            .collect();

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

    pub fn is_finished(&self) -> bool {
        self.time() >= self.length()
    }

    pub fn percentage(&self) -> f32 {
        self.running.as_secs_f32() / self.length().as_secs_f32()
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

    pub fn length(&self) -> Duration {
        self.last_note_end + self.leed_in
    }

    pub fn reset(&mut self) {
        self.running = Duration::ZERO;

        for state in self.track_states.iter_mut() {
            state.seen_events = 0;
        }
    }
}
