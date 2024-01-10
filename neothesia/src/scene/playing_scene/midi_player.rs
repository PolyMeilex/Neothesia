use midi_file::midly::{num::u4, MidiMessage};

use crate::{
    output_manager::OutputConnection,
    song::{PlayerConfig, Song},
};
use std::{
    collections::{HashSet, VecDeque},
    time::{Duration, Instant},
};

pub struct MidiPlayer {
    playback: midi_file::PlaybackState,
    output: OutputConnection,
    song: Song,
    play_along: PlayAlong,
}

impl MidiPlayer {
    pub fn new(
        output: OutputConnection,
        song: Song,
        user_keyboard_range: piano_math::KeyboardRange,
    ) -> Self {
        let mut player = Self {
            playback: midi_file::PlaybackState::new(
                Duration::from_secs(3),
                song.file.tracks.clone(),
            ),
            output,
            play_along: PlayAlong::new(user_keyboard_range),
            song,
        };
        // Let's reset programs,
        // for timestamp 0 most likely all programs will be 0, so this should clean any leftovers
        // from previous songs
        player.send_midi_programs_for_timestamp(&player.playback.time());
        player.update(Duration::ZERO);

        player
    }

    /// When playing: returns midi events
    ///
    /// When paused: returns None
    pub fn update(&mut self, delta: Duration) -> Vec<&midi_file::MidiEvent> {
        self.play_along.update();

        let events = self.playback.update(delta);

        events.iter().for_each(|event| {
            let config = &self.song.config.tracks[event.track_id];

            match config.player {
                PlayerConfig::Auto => {
                    self.output
                        .midi_event(u4::new(event.channel), event.message);
                }
                PlayerConfig::Human => {
                    // Let's play the sound, in case the user does not want it they can just set
                    // no-output output in settings
                    // TODO: Perhaps play on midi-in instead
                    self.output
                        .midi_event(u4::new(event.channel), event.message);
                    self.play_along
                        .midi_event(MidiEventSource::File, &event.message);
                }
                PlayerConfig::Mute => {}
            }
        });

        events
    }

    fn clear(&mut self) {
        self.output.stop_all();
    }
}

impl Drop for MidiPlayer {
    fn drop(&mut self) {
        self.clear();
    }
}

impl MidiPlayer {
    pub fn pause_resume(&mut self) {
        if self.playback.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn pause(&mut self) {
        self.clear();
        self.playback.pause();
    }

    pub fn resume(&mut self) {
        self.playback.resume();
        self.play_along.clear();
    }

    fn send_midi_programs_for_timestamp(&self, time: &Duration) {
        for (&channel, &p) in self.song.file.program_track.program_for_timestamp(time) {
            self.output.midi_event(
                u4::new(channel),
                midi_file::midly::MidiMessage::ProgramChange {
                    program: midi_file::midly::num::u7::new(p),
                },
            );
        }
    }

    fn set_time(&mut self, time: Duration) {
        self.playback.set_time(time);

        // Discard all of the events till that point
        let events = self.playback.update(Duration::ZERO);
        std::mem::drop(events);

        self.clear();
        self.send_midi_programs_for_timestamp(&time);
    }

    pub fn rewind(&mut self, delta: i64) {
        let mut time = self.playback.time();

        if delta < 0 {
            let delta = Duration::from_millis((-delta) as u64);
            time = time.saturating_sub(delta);
        } else {
            let delta = Duration::from_millis(delta as u64);
            time = time.saturating_add(delta);
        }

        self.set_time(time);
    }

    pub fn set_percentage_time(&mut self, p: f32) {
        self.set_time(Duration::from_secs_f32(
            (p * self.playback.lenght().as_secs_f32()).max(0.0),
        ));
    }

    pub fn percentage(&self) -> f32 {
        self.playback.percentage()
    }

    pub fn time_without_lead_in(&self) -> f32 {
        self.playback.time().as_secs_f32() - self.playback.leed_in().as_secs_f32()
    }

    pub fn is_paused(&self) -> bool {
        self.playback.is_paused()
    }
}

impl MidiPlayer {
    pub fn play_along(&self) -> &PlayAlong {
        &self.play_along
    }

    pub fn play_along_mut(&mut self) -> &mut PlayAlong {
        &mut self.play_along
    }
}

pub enum MidiEventSource {
    File,
    User,
}

#[derive(Debug)]
struct UserPress {
    timestamp: Instant,
    note_id: u8,
}

#[derive(Debug)]
pub struct PlayAlong {
    user_keyboard_range: piano_math::KeyboardRange,

    required_notes: HashSet<u8>,

    // List of user key press events that happened in last 500ms,
    // used for play along leeway logic
    user_pressed_recently: VecDeque<UserPress>,
}

impl PlayAlong {
    fn new(user_keyboard_range: piano_math::KeyboardRange) -> Self {
        Self {
            user_keyboard_range,
            required_notes: Default::default(),
            user_pressed_recently: Default::default(),
        }
    }

    fn update(&mut self) {
        // Instead of calling .elapsed() per item let's fetch `now` once, and substract it ourselfs
        let now = Instant::now();

        while let Some(item) = self.user_pressed_recently.front_mut() {
            let elapsed = now - item.timestamp;

            // If older than 500ms
            if elapsed.as_millis() > 500 {
                self.user_pressed_recently.pop_front();
            } else {
                // All subsequent items will by younger than front item, so we can break
                break;
            }
        }
    }

    fn user_press_key(&mut self, note_id: u8, active: bool) {
        let timestamp = Instant::now();

        if active {
            self.user_pressed_recently
                .push_back(UserPress { timestamp, note_id });
            self.required_notes.remove(&note_id);
        }
    }

    fn file_press_key(&mut self, note_id: u8, active: bool) {
        if active {
            if let Some((id, _)) = self
                .user_pressed_recently
                .iter()
                .enumerate()
                .find(|(_, item)| item.note_id == note_id)
            {
                self.user_pressed_recently.remove(id);
            } else {
                self.required_notes.insert(note_id);
            }
        } else {
            self.required_notes.remove(&note_id);
        }
    }

    fn press_key(&mut self, src: MidiEventSource, note_id: u8, active: bool) {
        if !self.user_keyboard_range.contains(note_id) {
            return;
        }

        match src {
            MidiEventSource::User => self.user_press_key(note_id, active),
            MidiEventSource::File => self.file_press_key(note_id, active),
        }
    }

    pub fn midi_event(&mut self, source: MidiEventSource, message: &MidiMessage) {
        match message {
            MidiMessage::NoteOn { key, .. } => self.press_key(source, key.as_int(), true),
            MidiMessage::NoteOff { key, .. } => self.press_key(source, key.as_int(), false),
            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.required_notes.clear()
    }

    pub fn are_required_keys_pressed(&self) -> bool {
        self.required_notes.is_empty()
    }
}
