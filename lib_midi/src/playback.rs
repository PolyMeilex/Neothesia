use std::{collections::HashSet, time::Duration};

use midly::MidiMessage;

use crate::{MidiEvent, MidiTrack};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ActiveNote {
    pub key: u8,
    pub channel: u8,
}

#[derive(Debug, Clone)]
pub struct PlaybackState {
    is_paused: bool,
    running: Duration,
    leed_in: Duration,
    seen_events: usize,

    first_note_start: Duration,
    last_note_end: Duration,

    active_notes: HashSet<ActiveNote>,
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
            is_paused: false,
            running: Duration::ZERO,
            leed_in,
            seen_events: 0,

            first_note_start,
            last_note_end,

            active_notes: Default::default(),
        }
    }

    pub fn update(&mut self, track: &MidiTrack, delta: Duration) -> Option<Vec<MidiEvent>> {
        if !self.is_paused {
            self.running += delta;

            let events = track
                .events
                .iter()
                .skip(self.seen_events)
                .filter(|event| event.timestamp + self.leed_in <= self.running)
                .map(|event| {
                    let event = event.clone();
                    self.seen_events += 1;
                    event
                })
                .inspect(|event| match event.message {
                    MidiMessage::NoteOn { key, .. } => {
                        self.active_notes.insert(ActiveNote {
                            key: key.as_int(),
                            channel: event.channel,
                        });
                    }
                    MidiMessage::NoteOff { key, .. } => {
                        self.active_notes.remove(&ActiveNote {
                            key: key.as_int(),
                            channel: event.channel,
                        });
                    }
                    _ => {}
                })
                .collect();

            Some(events)
        } else {
            None
        }
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

    pub fn active_notes(&self) -> &HashSet<ActiveNote> {
        &self.active_notes
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
        self.active_notes.clear();
    }
}
