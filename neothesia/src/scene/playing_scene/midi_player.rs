use midi_file::midly::{MidiMessage, num::u4};

use crate::{
    output_manager::OutputConnection,
    song::{ChannelConfig, ChannelMode, Song},
};
use neothesia_core::piano_layout;
use std::{
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

pub struct MidiPlayer {
    playback: midi_file::PlaybackState,
    output: OutputConnection,
    song: Song,
    play_along: PlayAlong,
    separate_channels: bool,
}

impl MidiPlayer {
    pub fn new(
        output: OutputConnection,
        song: Song,
        user_keyboard_range: piano_layout::KeyboardRange,
        separate_channels: bool,
    ) -> Self {
        let mut player = Self {
            playback: midi_file::PlaybackState::new(
                Duration::from_secs(3),
                song.file.tracks.clone(),
            ),
            output,
            play_along: PlayAlong::new(user_keyboard_range),
            song,
            separate_channels,
        };
        // Let's reset programs,
        // for timestamp 0 most likely all programs will be 0, so this should clean any leftovers
        // from previous songs
        player.send_midi_programs_for_timestamp(&player.playback.time());
        player.update(Duration::ZERO);

        player
    }

    /// Get the channel config for a given event channel from the track configuration.
    /// Returns a default config (Listen mode, active) if the channel is not found.
    fn get_channel_config(track_config: &crate::song::TrackConfig, channel: u8) -> ChannelConfig {
        track_config.channels
            .iter()
            .find(|cc| cc.channel == channel)
            .cloned()
            .unwrap_or(ChannelConfig {
                channel,
                mode: ChannelMode::Listen,
                active: true,
                interactive: channel != 9, // Channel 9 is drums - not interactive
            })
    }

    pub fn song(&self) -> &Song {
        &self.song
    }

    pub fn song_mut(&mut self) -> &mut Song {
        &mut self.song
    }

    /// When playing: returns midi events
    ///
    /// When paused: returns None
    pub fn update(&mut self, delta: Duration) -> Vec<&midi_file::MidiEvent> {
        self.play_along.update();

        // Collect triggered notes before borrowing events
        let triggered_notes: std::collections::HashMap<u8, std::collections::HashSet<u8>> = 
            self.play_along.user_triggered_notes.clone();

        let all_events: Vec<_> = self.playback.update(delta);

        // Filter events based on channel state for visuals (skip inactive channels)
        let events: Vec<_> = all_events
            .iter()
            .filter(|event| {
                let config = &self.song.config.tracks[event.track_id];
                let channel = if self.separate_channels {
                    event.track_color_id as u8
                } else {
                    event.channel
                };
                let channel_config = Self::get_channel_config(config, channel);

                // Inactive channels are filtered out entirely (no visual, no audio, no waiting)
                channel_config.active
            })
            .cloned()
            .collect();

        // Process audio for each event based on its channel configuration
        for event in &all_events {
            let config = &self.song.config.tracks[event.track_id];
            let channel = if self.separate_channels {
                event.track_color_id as u8
            } else {
                event.channel
            };
            let channel_config = Self::get_channel_config(config, channel);

            if !channel_config.active {
                // Channel deactivated: skip entirely (no audio, no visuals, no waiting)
                continue;
            }

            // Update play-along state for file notes (for progression and wait mode logic)
            // Only interactive channels participate in wait mode - non-interactive channels
            // (like drums, channel 9) play automatically without requiring user input
            if channel_config.interactive {
                self.play_along.midi_event(MidiEventSource::File, &event.message);
            }

            // Process audio based on mode
            match channel_config.mode {
                ChannelMode::Listen => {
                    // Play MIDI audio
                    self.output
                        .midi_event(u4::new(channel), event.message);
                }
                ChannelMode::Assist => {
                    // Play MIDI audio only if user has already triggered the note
                    // (to avoid double-playing and only play human-triggered notes)
                    if Self::should_skip_event_with_set(&triggered_notes, channel, &event.message) {
                        self.output
                            .midi_event(u4::new(channel), event.message);
                    }
                }
                ChannelMode::Alone => {
                    // Silence MIDI audio (always) - keyboard audio handled separately
                    // No MIDI file audio output in Alone mode
                }
            }
        }

        events
    }

    /// Helper to check if an event should be skipped, using a pre-collected set of triggered notes.
    fn should_skip_event_with_set(triggered_notes: &std::collections::HashMap<u8, std::collections::HashSet<u8>>, 
                                  channel: u8,
                                  message: &midi_file::midly::MidiMessage) -> bool {
        match message {
            midi_file::midly::MidiMessage::NoteOn { key, .. } => {
                let note_id = key.as_int();
                triggered_notes.get(&channel)
                    .map(|notes| notes.contains(&note_id))
                    .unwrap_or(false)
            }
            _ => false,
        }
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

    pub fn set_time(&mut self, time: Duration) {
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

    pub fn percentage_to_time(&self, p: f32) -> Duration {
        Duration::from_secs_f32((p * self.playback.length().as_secs_f32()).max(0.0))
    }

    pub fn time_to_percentage(&self, time: &Duration) -> f32 {
        time.as_secs_f32() / self.playback.length().as_secs_f32()
    }

    pub fn set_percentage_time(&mut self, p: f32) {
        self.set_time(self.percentage_to_time(p));
    }

    pub fn leed_in(&self) -> &Duration {
        self.playback.leed_in()
    }

    pub fn length(&self) -> Duration {
        self.playback.length()
    }

    pub fn percentage(&self) -> f32 {
        self.playback.percentage()
    }

    pub fn is_finished(&self) -> bool {
        self.playback.is_finished()
    }

    pub fn time(&self) -> Duration {
        self.playback.time()
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

type NoteId = u8;

#[derive(Debug, Default)]
struct PlayerStats {
    /// User notes that expired, or were simply wrong
    wrong_notes: usize,
    /// List of deltas of notes played early
    played_early: Vec<Duration>,
    /// List of deltas of notes played late
    played_late: Vec<Duration>,
}

impl PlayerStats {
    #[allow(unused)]
    fn timing_acurracy(&self) -> f64 {
        let all = self.played_early.len() + self.played_late.len();
        let early_count = self.count_too_early();
        let late_count = self.count_too_late();
        (early_count + late_count) as f64 / all as f64
    }

    fn count_too_early(&self) -> usize {
        // 500 is the same as expire time, so this does not make much sense, but we can chooses
        // better threshold later down the line
        Self::count_with_threshold(&self.played_early, Duration::from_millis(500))
    }

    fn count_too_late(&self) -> usize {
        // 160 to forgive touching the bottom
        Self::count_with_threshold(&self.played_late, Duration::from_millis(160))
    }

    fn count_with_threshold(events: &[Duration], threshold: Duration) -> usize {
        events
            .iter()
            .filter(|delta| **delta > threshold)
            .fold(0, |n, _| n + 1)
    }
}

#[derive(Debug)]
pub struct NotePress {
    timestamp: Instant,
}

#[derive(Debug)]
pub struct PlayAlong {
    user_keyboard_range: piano_layout::KeyboardRange,

    /// Notes required to proggres further in the song
    required_notes: HashMap<NoteId, NotePress>,
    /// List of user key press events that happened in last 500ms,
    /// used for play along leeway logic
    user_pressed_recently: HashMap<NoteId, NotePress>,
    /// File notes that had NoteOn event, but no NoteOff yet
    in_proggres_file_notes: HashSet<NoteId>,
    /// Notes that were already played by the user in wait mode (to avoid double-trigger)
    /// Tracks per channel: channel -> set of note_ids
    user_triggered_notes: HashMap<u8, HashSet<NoteId>>,

    stats: PlayerStats,
}
impl PlayAlong {
    fn new(user_keyboard_range: piano_layout::KeyboardRange) -> Self {
        Self {
            user_keyboard_range,
            required_notes: Default::default(),
            user_pressed_recently: Default::default(),
            in_proggres_file_notes: Default::default(),
            user_triggered_notes: Default::default(),
            stats: PlayerStats::default(),
        }
    }

    fn update(&mut self) {
        // Instead of calling .elapsed() per item let's fetch `now` once, and subtract it ourselves
        let now = Instant::now();
        let threshold = Duration::from_millis(500);

        // Track the count of items before retain
        let count_before = self.user_pressed_recently.len();

        // Retain only the items that are within the threshold
        self.user_pressed_recently
            .retain(|_, item| now.duration_since(item.timestamp) <= threshold);

        self.stats.wrong_notes += count_before - self.user_pressed_recently.len();
    }

    fn user_press_key(&mut self, note_id: u8, active: bool) {
        let timestamp = Instant::now();

        if active {
            // Check if note has already been played by a file
            if let Some(required_press) = self.required_notes.remove(&note_id) {
                self.stats
                    .played_late
                    .push(timestamp.duration_since(required_press.timestamp));
            } else {
                // This note was not played by file yet, place it in recents
                let got_replaced = self
                    .user_pressed_recently
                    .insert(note_id, NotePress { timestamp })
                    .is_some();

                if got_replaced {
                    self.stats.wrong_notes += 1
                }
            }
        }
    }

    fn file_press_key(&mut self, note_id: u8, active: bool) {
        let timestamp = Instant::now();
        if active {
            // Check if note got pressed earlier 500ms (user_pressed_recently)
            if let Some(press) = self.user_pressed_recently.remove(&note_id) {
                self.stats
                    .played_early
                    .push(timestamp.duration_since(press.timestamp));
            } else {
                // Player never pressed that note, let it reach required_notes

                // Ignore overlapping notes
                if self.in_proggres_file_notes.contains(&note_id) {
                    return;
                }

                self.required_notes.insert(note_id, NotePress { timestamp });
            }

            self.in_proggres_file_notes.insert(note_id);
        } else {
            self.in_proggres_file_notes.remove(&note_id);
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
        self.required_notes.clear();
        self.user_pressed_recently.clear();
        self.in_proggres_file_notes.clear();
        self.user_triggered_notes.clear();
    }

    pub fn are_required_keys_pressed(&self) -> bool {
        self.required_notes.is_empty()
    }

    pub fn get_required_notes(&self) -> &HashMap<NoteId, NotePress> {
        &self.required_notes
    }

    /// Check if a user key press matches a required note in wait mode.
    /// Returns true if the note is required, without consuming it.
    pub fn is_required_note(&self, note_id: u8) -> bool {
        self.required_notes.contains_key(&note_id)
    }

    /// Mark a note as having been triggered by the user in wait mode.
    pub fn mark_note_as_triggered(&mut self, channel: u8, note_id: u8) {
        self.user_triggered_notes
            .entry(channel)
            .or_default()
            .insert(note_id);
    }

    /// Check if a note was already triggered by the user for a specific channel.
    pub fn was_note_triggered(&self, channel: u8, note_id: u8) -> bool {
        self.user_triggered_notes
            .get(&channel)
            .map(|notes| notes.contains(&note_id))
            .unwrap_or(false)
    }
}
