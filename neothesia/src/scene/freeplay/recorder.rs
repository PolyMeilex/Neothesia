use std::{
    collections::HashSet,
    fmt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use midi_file::midly::{
    Format, Header, MetaMessage, MidiMessage, Smf, Timing, TrackEvent, TrackEventKind,
};
use neothesia_core::render::{NoteLabels, WaterfallRenderer};

use crate::{scene::playing_scene::midi_player::MidiPlayer, song::Song};

#[derive(Debug, Eq, PartialEq, thiserror::Error)]
pub enum RecorderError {
    #[error("No note events recorded")]
    NoNotesFound,
    #[error("Failed to write MIDI file")]
    Write,
    #[error("{0}")]
    MidiFileParse(String),
}

#[derive(Default, Debug)]
pub enum RecorderStatus {
    #[default]
    Idle,
    RecordingFinished(Duration),
    Saved(PathBuf),
    Error(RecorderError),
}

impl fmt::Display for RecorderStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => {}
            Self::RecordingFinished(duration) => {
                write!(f, "Recorded {:.1}s", duration.as_secs_f32())?;
            }
            Self::Error(err) => {
                write!(f, "{err}")?;
            }
            Self::Saved(path) => {
                write!(f, "Saved recording to {}", path.display())?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct RecordedMidiEvent {
    timestamp: Duration,
    channel: u8,
    message: MidiMessage,
}

pub struct RecordingInProgressState {
    started_at: Instant,
    events: Vec<RecordedMidiEvent>,
    active_notes: HashSet<(u8, u8)>,
}

pub struct RecordedTake {
    duration: Duration,
    events: Vec<RecordedMidiEvent>,
}

#[derive(Default)]
enum RecorderState {
    #[default]
    Idle,
    Recording(RecordingInProgressState),
    Recorded(RecordedTake),
}

#[derive(Default)]
pub struct FreeplayRecorder {
    state: RecorderState,
}

pub struct PreviewState {
    pub player: MidiPlayer,
    pub waterfall: WaterfallRenderer,
    pub note_labels: Option<NoteLabels>,
}

impl FreeplayRecorder {
    const TICKS_PER_BEAT: u16 = 480;
    const TEMPO_MICROS_PER_BEAT: u32 = 500_000;
    const TICKS_PER_SECOND: f64 =
        Self::TICKS_PER_BEAT as f64 * 1_000_000.0 / Self::TEMPO_MICROS_PER_BEAT as f64;

    pub fn is_recording(&self) -> bool {
        matches!(self.state, RecorderState::Recording(_))
    }

    pub fn has_note_events(&self) -> bool {
        match &self.state {
            // Preview/export requires at least one played note, not just release/control data.
            RecorderState::Recorded(recorded_take) => recorded_take
                .events
                .iter()
                .any(|event| matches!(event.message, MidiMessage::NoteOn { .. })),
            RecorderState::Idle | RecorderState::Recording(_) => false,
        }
    }

    pub fn start(&mut self) {
        self.state = RecorderState::Recording(RecordingInProgressState {
            started_at: Instant::now(),
            events: Vec::new(),
            active_notes: HashSet::new(),
        });
    }

    pub fn stop(&mut self) {
        let state = std::mem::take(&mut self.state);
        let RecorderState::Recording(mut in_progress) = state else {
            return;
        };

        let stop_time = in_progress.started_at.elapsed();
        Self::finish_active_notes(&mut in_progress, stop_time);
        self.state = RecorderState::Recorded(RecordedTake {
            duration: stop_time,
            events: in_progress.events,
        });
    }

    pub fn duration(&self) -> Duration {
        match &self.state {
            RecorderState::Idle => Duration::ZERO,
            RecorderState::Recording(state) => state.started_at.elapsed(),
            RecorderState::Recorded(recorded_take) => recorded_take.duration,
        }
    }

    pub fn push_event(&mut self, channel: u8, message: MidiMessage) {
        let RecorderState::Recording(in_progress) = &mut self.state else {
            return;
        };

        let timestamp = in_progress.started_at.elapsed();
        in_progress.events.push(RecordedMidiEvent {
            timestamp,
            channel,
            message,
        });

        match message {
            MidiMessage::NoteOn { key, .. } => {
                in_progress.active_notes.insert((channel, key.as_int()));
            }
            MidiMessage::NoteOff { key, .. } => {
                in_progress.active_notes.remove(&(channel, key.as_int()));
            }
            _ => {}
        }
    }

    pub fn save_to_path(smf: Smf<'static>, path: &Path) -> Result<(), RecorderError> {
        let mut bytes = Vec::new();

        if smf.write_std(&mut bytes).is_err() {
            return Err(RecorderError::Write);
        }

        if std::fs::write(path, bytes).is_err() {
            return Err(RecorderError::Write);
        }

        Ok(())
    }

    pub fn to_song(&self) -> Result<Song, RecorderError> {
        let smf = self.to_smf()?;
        let midi = midi_file::MidiFile::from_smf("freeplay-recording.mid", smf)
            .map_err(RecorderError::MidiFileParse)?;
        Ok(Song::new(midi))
    }

    fn finish_active_notes(in_progress: &mut RecordingInProgressState, timestamp: Duration) {
        let mut active_notes: Vec<_> = in_progress.active_notes.drain().collect();
        active_notes.sort_unstable();

        for (channel, key) in active_notes {
            in_progress.events.push(RecordedMidiEvent {
                timestamp,
                channel,
                message: MidiMessage::NoteOff {
                    key: key.into(),
                    vel: 0.into(),
                },
            });
        }
    }

    pub fn to_smf(&self) -> Result<Smf<'static>, RecorderError> {
        if !self.has_note_events() {
            return Err(RecorderError::NoNotesFound);
        }

        let events = match &self.state {
            RecorderState::Recorded(recorded_take) => recorded_take.events.as_slice(),
            RecorderState::Idle | RecorderState::Recording(_) => &[],
        };

        let mut track = vec![
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::Tempo(Self::TEMPO_MICROS_PER_BEAT.into())),
            },
            TrackEvent {
                delta: 0.into(),
                kind: TrackEventKind::Meta(MetaMessage::TimeSignature(4, 2, 24, 8)),
            },
        ];

        let mut previous_ticks = 0u32;
        for event in events {
            let current_ticks = Self::duration_to_ticks(event.timestamp);
            let delta_ticks = current_ticks.saturating_sub(previous_ticks);
            previous_ticks = current_ticks;

            track.push(TrackEvent {
                delta: delta_ticks.into(),
                kind: TrackEventKind::Midi {
                    channel: event.channel.into(),
                    message: event.message,
                },
            });
        }

        track.push(TrackEvent {
            delta: 1.into(),
            kind: TrackEventKind::Meta(MetaMessage::EndOfTrack),
        });

        Ok(Smf {
            header: Header {
                format: Format::SingleTrack,
                timing: Timing::Metrical(Self::TICKS_PER_BEAT.into()),
            },
            tracks: vec![track],
        })
    }

    fn duration_to_ticks(duration: Duration) -> u32 {
        (duration.as_secs_f64() * Self::TICKS_PER_SECOND).round() as u32
    }
}

#[cfg(test)]
mod freeplay_recorder_tests {
    use super::*;

    #[test]
    fn restarting_recording_discards_previous_take_and_resets_event_count() {
        let mut recorder = FreeplayRecorder::default();

        recorder.start();
        recorder.push_event(
            0,
            MidiMessage::NoteOn {
                key: 60.into(),
                vel: 100.into(),
            },
        );
        recorder.stop();

        assert!(recorder.has_note_events());

        recorder.start();

        assert!(recorder.is_recording());
        assert!(!recorder.has_note_events());
    }

    #[test]
    fn pedal_only_recording_is_rejected_for_preview_song() {
        let mut recorder = FreeplayRecorder::default();

        recorder.start();
        recorder.push_event(
            0,
            MidiMessage::Controller {
                controller: 64.into(),
                value: 127.into(),
            },
        );
        recorder.push_event(
            0,
            MidiMessage::Controller {
                controller: 64.into(),
                value: 0.into(),
            },
        );
        recorder.stop();

        assert!(!recorder.has_note_events());
        let error = recorder
            .to_song()
            .expect_err("pedal-only recordings should not create preview songs");
        assert_eq!(error, RecorderError::NoNotesFound);
    }

    #[test]
    fn note_off_only_recording_is_rejected_for_preview_song() {
        let mut recorder = FreeplayRecorder::default();

        recorder.start();
        recorder.push_event(
            0,
            MidiMessage::NoteOff {
                key: 60.into(),
                vel: 0.into(),
            },
        );
        recorder.stop();

        assert!(!recorder.has_note_events());
        let error = recorder
            .to_song()
            .expect_err("note-off-only recordings should not create preview songs");
        assert_eq!(error, RecorderError::NoNotesFound);
    }
}
