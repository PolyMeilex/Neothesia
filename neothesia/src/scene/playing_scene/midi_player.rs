use crate::{
    output_manager::OutputConnection,
    song::{PlayerConfig, Song},
};
use midi_file::midly::{num::u4, MidiMessage};
use neothesia_core::gamesave::{SavedStats, SongStats};
use std::time::SystemTime;
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

pub struct MidiPlayer {
    playback: midi_file::PlaybackState,
    output: OutputConnection,
    song: Song,
    play_along: PlayAlong,
}

pub struct NoteStats {
    song_name: String,
    notes_missed: usize,
    notes_hit: usize,
    wrong_notes: usize,
    note_durations: Vec<NoteDurations>,
}
#[derive(Debug)]
pub struct NoteDurations {
    user_note_dur: usize,
    file_note_dur: usize,
}
impl Default for NoteStats {
    fn default() -> Self {
        NoteStats {
            song_name: String::new(),
            notes_missed: 0,
            notes_hit: 0,
            wrong_notes: 0,
            note_durations: Vec::new(),
        }
    }
}

impl Default for NoteDurations {
    fn default() -> Self {
        NoteDurations {
            user_note_dur: 0,
            file_note_dur: 0,
        }
    }
}

impl MidiPlayer {
    pub fn new(
        output: OutputConnection,
        song: Song,
        user_keyboard_range: piano_math::KeyboardRange,
    ) -> Self {
        let mut user_stats = NoteStats::default();

        user_stats.song_name = Song::get_clean_songname(song.file.name.clone());

        let mut player = Self {
            playback: midi_file::PlaybackState::new(
                Duration::from_secs(3),
                song.file.tracks.clone(),
            ),
            output,
            play_along: PlayAlong::new(user_keyboard_range, user_stats),
            song,
        };
        // Let's reset programs,
        // for timestamp 0 most likely all programs will be 0, so this should clean any leftovers
        // from previous songs
        player.send_midi_programs_for_timestamp(&player.playback.time());
        player.update(Duration::ZERO);

        player
    }
    pub fn on_finish<F>(&mut self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.play_along.set_on_finish(callback);
    }
    pub fn song(&self) -> &Song {
        &self.song
    }

    /// When playing: returns midi events
    ///
    /// When paused: returns None
    pub fn update(&mut self, delta: Duration) -> Vec<&midi_file::MidiEvent> {
        self.play_along.update();

        let playback_time = self.playback.time();
        let playback_length = self.playback.length();

        let events = self.playback.update(delta);

        events.iter().for_each(|event| {
            let config = &self.song.config.tracks[event.track_id];

            match config.player {
                PlayerConfig::Auto => {
                    self.output
                        .midi_event(u4::new(event.channel), event.message);
                }
                PlayerConfig::Human => {
                    self.output
                        .midi_event(u4::new(event.channel), event.message);
                    self.play_along
                        .midi_event(MidiEventSource::File, &event.message);
                }
                PlayerConfig::Mute => {}
            }
        });

        // Check if the song has finished based on the playback time
        if playback_time >= playback_length {
            self.play_along.finished();
        }

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

#[derive(Debug)]
struct UserPress {
    timestamp: Instant,
    note_id: u8,
    time_key_up: Option<Instant>,
    occurrence: usize,
}

pub struct PlayAlong {
    user_keyboard_range: piano_math::KeyboardRange,

    required_notes: Vec<UserPress>,
    finished: bool,
    // List of user key press events that happened in last 500ms,
    // used for play along leeway logic
    user_pressed_recently: Vec<UserPress>,
    user_stats: NoteStats, // struct to finalize the stats log

    user_notes: Vec<UserPress>,     // log all user notes to get durrations
    file_notes: Vec<UserPress>,     // log all file notes to compare against user
    occurrence: HashMap<u8, usize>, // Keeping user to file log incremental pointer rewind immune
    on_finish: Option<Box<dyn Fn()>>,
}

impl PlayAlong {
    fn new(user_keyboard_range: piano_math::KeyboardRange, user_stats: NoteStats) -> Self {
        Self {
            user_keyboard_range,
            required_notes: Default::default(),
            user_pressed_recently: Default::default(),

            occurrence: HashMap::new(),
            finished: Default::default(),

            user_notes: Default::default(),
            file_notes: Default::default(),
            on_finish: None,
            user_stats,
        }
    }
    pub fn set_on_finish<F>(&mut self, callback: F)
    where
        F: Fn() + 'static,
    {
        self.on_finish = Some(Box::new(callback));
    }

    fn update(&mut self) {
        let now = Instant::now();
        let threshold = Duration::from_millis(500);

        // Track the count of items before retain
        let count_before = self.user_pressed_recently.len();

        // Retain only the items that are within the threshold
        self.user_pressed_recently.retain(|item| {
            let elapsed = now - item.timestamp;
            elapsed <= threshold
        });

        // Calculate the count of deleted items,
        // Either pressed extremely too earlier than should have, or a wrong note, both cases is wrong note, we don't sub these.
        let count_deleted = count_before - self.user_pressed_recently.len();
        if count_deleted > 0 {
            self.user_stats.wrong_notes += count_deleted;
        }
    }

    fn user_press_key(&mut self, note_id: u8, active: bool) {
        let timestamp = Instant::now();
        let occurrence = self.occurrence.entry(note_id).or_insert(0);

        if active {
            // Check if note_id has reached required_notes, then remove it now,

            if let Some(index) = self
                .required_notes
                .iter()
                .position(|item| item.note_id == note_id)
            {
                if timestamp
                    .duration_since(self.required_notes[index].timestamp)
                    .as_millis()
                    > 160
                {
                    //160 to forgive touching the bottom

                    self.user_stats.notes_missed += 1;
                } else {
                    self.user_stats.notes_hit += 1;
                }
                self.required_notes.remove(index);

                self.user_notes.push(UserPress {
                    timestamp,
                    note_id,
                    occurrence: *occurrence,
                    time_key_up: None,
                });
            } else {
                // Haven't reached required_notes yet, place a possible later validation in 'user_pressed_recently' / file_press_key()
                if let Some(item) = self
                    .user_pressed_recently
                    .iter_mut()
                    .find(|item| item.note_id == note_id)
                {
                    // already exists, update timestamp
                    item.timestamp = timestamp;
                    self.user_stats.wrong_notes += 1;
                } else {
                    // Not found, push a new UserPress
                    self.user_pressed_recently.push(UserPress {
                        timestamp,
                        note_id,
                        occurrence: *occurrence,
                        time_key_up: None,
                    });
                }
            }
        } else {
            // Update user_notes log time_key_up
            if let Some(item) = self
                .user_notes
                .iter_mut()
                .rev()
                .find(|item| item.note_id == note_id && item.occurrence == *occurrence)
            {
                item.time_key_up = Some(Instant::now());
            }
        }
    }

    fn file_press_key(&mut self, note_id: u8, active: bool) {
        let occurrence = self.occurrence.entry(note_id).or_insert(0);
        let timestamp = Instant::now();
        if active {
            *occurrence += 1;

            // Check if note got pressed earlier 500ms (user_pressed_recently)
            if let Some(item) = self
                .user_pressed_recently
                .iter()
                .find(|item| item.note_id == note_id)
            {
                // Note was pressed earlier, remove it from user_pressed_recently
                self.user_stats.notes_hit += 1;

                // log user_note by user_pressed_recently.timestamp as keydown value, update occurence
                self.user_notes.push(UserPress {
                    timestamp: item.timestamp,
                    note_id,
                    occurrence: *occurrence,
                    time_key_up: item.time_key_up,
                });
                self.user_pressed_recently
                    .retain(|item| item.note_id != note_id);
            } else {
                // Player never pressed that note, let it reach required_notes, check if note_id already exists in required_notes,  update timestamp else push.
                // Catch possible clone-note velocity overlay, update the new occurence and exit the function

                if let Some(item) = self
                    .file_notes
                    .iter_mut()
                    .find(|item| item.note_id == note_id && item.time_key_up.is_none())
                {
                    item.occurrence = *occurrence;
                    return; //  Everything bellow already done before by its clone
                }
                if let Some(user_press) = self
                    .required_notes
                    .iter_mut()
                    .find(|item| item.note_id == note_id)
                {
                    // Update the timestamp of the existing note
                    user_press.timestamp = timestamp;
                } else {
                    self.required_notes.push(UserPress {
                        timestamp,
                        note_id,
                        occurrence: *occurrence,
                        time_key_up: None,
                    });
                }
            }

            // Log the note
            self.file_notes.push(UserPress {
                timestamp,
                note_id,
                occurrence: *occurrence, // Set the occurrence count
                time_key_up: None,
            });
        } else {
            // update time_key_up
            if let Some(item) = self
                .file_notes
                .iter_mut()
                .rev()
                .find(|item| item.note_id == note_id && item.occurrence == *occurrence)
            {
                item.time_key_up = Some(timestamp);
            }
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
        // Remove from the file log, notes that left pressed down with no key up yet (rewinding a non-played part)
        self.file_notes.retain(|item| item.time_key_up.is_some());
        self.user_pressed_recently.clear();
    }

    pub fn finished(&mut self) {
        if !self.finished {
            // Loop through user_notes and file_notes and match entries with the same occurrence[note] = num
            for user_note in &self.user_notes {
                for file_note in &self.file_notes {
                    if user_note.occurrence == file_note.occurrence
                        && user_note.note_id == file_note.note_id
                    {
                        // Subtract timestamp from time_key_up to get total seconds
                        let user_note_dur = match (user_note.timestamp, user_note.time_key_up) {
                            (start, Some(end)) => end.duration_since(start).as_secs(),
                            _ => 0,
                        };
                        let file_note_dur = match (file_note.timestamp, file_note.time_key_up) {
                            (start, Some(end)) => end.duration_since(start).as_secs(),
                            _ => 0,
                        };

                        // Add this information to user_stats.note_durations
                        let note_duration = NoteDurations {
                            user_note_dur: user_note_dur as usize,
                            file_note_dur: file_note_dur as usize,
                        };
                        self.user_stats.note_durations.push(note_duration);
                    }
                }
            }

            //  Loop through user_stats.note_durations items, compare user_note_dur to file_note_dur
            let mut correct_note_times = 0;
            let mut wrong_note_times = 0;
            // make it relaxed, Lower Bound: 87% of the file's note duration, Upper Bound: 108% of the file's note duration.
            for duration in &self.user_stats.note_durations {
                // Calculate the lower and upper bounds for a "correct" duration
                let lower_bound = duration.file_note_dur as f64 * 0.87;
                let upper_bound = duration.file_note_dur as f64 * 1.08;

                // Increment correctNoteTimes if it is within the bounds, otherwise increment wrongNoteTimes
                if (duration.user_note_dur as f64) >= lower_bound
                    && (duration.user_note_dur as f64) <= upper_bound
                {
                    correct_note_times += 1;
                } else {
                    wrong_note_times += 1;
                }
            }

            // Save only if the user pressed something, it wasn't a full rewind OR [AUTO]

            if self.user_stats.notes_hit
                + self.user_stats.notes_missed
                + self.user_stats.wrong_notes
                > 0
            {
                let mut saved_stats = SavedStats::load().unwrap_or_default();

                // Create the new stats object
                let new_stats = SongStats {
                    song_name: self.user_stats.song_name.clone(),
                    correct_note_times,
                    wrong_note_times,
                    notes_missed: self.user_stats.notes_missed as u32,
                    notes_hit: self.user_stats.notes_hit as u32,
                    wrong_notes: self.user_stats.wrong_notes as u32,
                    date: SystemTime::now(),
                };
                //
                // Push the new stats object to the existing SavedStats
                saved_stats.songs.push(new_stats);

                // Save the modified SavedStats object
                saved_stats.save();
            }

            // better save right here keeping things simple, since stats could be loaded from song list when select folder for a file list is implemented

            if let Some(callback) = &self.on_finish {
                callback(); // Call on finish callback
            }
            self.finished = true;
        }
    }
    pub fn are_required_keys_pressed(&self) -> bool {
        self.required_notes.is_empty()
    }
}
