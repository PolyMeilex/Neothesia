//! Sheet music notation layout and rendering for Neothesia.
//!
//! Converts MIDI note data into a structured score representation
//! and renders it as a scrolling grand staff (treble + bass clefs).

mod layout;
pub mod render;

use midi_file::MidiTrack;
use std::time::Duration;

// ── Musical types ──────────────────────────────────────────────────────

/// A pitch as a MIDI note number (21 = A0, 60 = C4, 108 = C8).
pub type MidiPitch = u8;

/// Diatonic position on a 5-line staff. Each integer is one diatonic step
/// (line OR space): lines are at even positions, spaces at odd. 0 = bottom
/// line, 8 = top line. Treble: 0 = E4, 4 = B4 (middle line), 8 = F5.
/// Bass: 0 = G2, 4 = D3 (middle line), 8 = A3. Values can go above 8
/// (ledger lines above) or below 0 (ledger lines below).
pub type StaffPos = i8;

/// Diatonic position of the middle line (B4 on treble, D3 on bass).
pub const MIDDLE_LINE_POS: StaffPos = 4;

/// A musical note duration expressed as a fraction of a whole note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhythmicValue {
    Whole,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}

impl RhythmicValue {
    /// Beats this note occupies (assuming quarter-note beat).
    pub const fn beats(self) -> f32 {
        match self {
            RhythmicValue::Whole => 4.0,
            RhythmicValue::Half => 2.0,
            RhythmicValue::Quarter => 1.0,
            RhythmicValue::Eighth => 0.5,
            RhythmicValue::Sixteenth => 0.25,
            RhythmicValue::ThirtySecond => 0.125,
        }
    }

    /// Beats this note occupies, accounting for the augmentation dot (1.5×).
    pub fn beats_with_dot(self, dotted: bool) -> f32 {
        self.beats() * if dotted { 1.5 } else { 1.0 }
    }

    /// Wall-clock duration of this rhythmic value at a given beat duration.
    pub fn duration(self, dotted: bool, beat_duration: Duration) -> Duration {
        beat_duration.mul_f32(self.beats_with_dot(dotted))
    }
}

/// A single notehead within a chord.
#[derive(Debug, Clone)]
pub struct Notehead {
    pub staff_pos: StaffPos,
    /// The color index from the MIDI track, for coloring.
    pub color_id: usize,
    /// Absolute start time of this note.
    pub start_time: Duration,
    /// Absolute end time of this note.
    pub end_time: Duration,
}

/// One or more noteheads sharing a stem, rhythmic value, and x position.
/// A single note is a chord with one head. The stem is only drawn when
/// `rhythmic_value` is not [`RhythmicValue::Whole`].
#[derive(Debug, Clone)]
pub struct Chord {
    pub heads: Vec<Notehead>,
    pub rhythmic_value: RhythmicValue,
    pub dotted: bool,
    pub stem_direction: StemDirection,
    /// Onset time of the chord within the score.
    pub start_time: Duration,
}

/// Stem direction for a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemDirection {
    Up,
    Down,
}

/// A rest element on the staff.
#[derive(Debug, Clone)]
pub struct Rest {
    pub rhythmic_value: RhythmicValue,
    pub dotted: bool,
    /// Onset time of the rest within the score.
    pub start_time: Duration,
}

/// A musical element on a staff: either a chord or a rest.
#[derive(Debug, Clone)]
pub enum StaffElement {
    Chord(Chord),
    Rest(Rest),
}

impl StaffElement {
    pub fn start_time(&self) -> Duration {
        match self {
            StaffElement::Chord(c) => c.start_time,
            StaffElement::Rest(r) => r.start_time,
        }
    }
}

/// Which clef a staff uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clef {
    Treble,
    Bass,
}

impl Clef {
    /// Diatonic index (`7*octave + note_index`) of the bottom line of this
    /// staff. Treble bottom line is E4 = `5*7 + 2`; bass bottom line is
    /// G2 = `3*7 + 4`.
    const fn bottom_diatonic(self) -> i16 {
        match self {
            Clef::Treble => 37,
            Clef::Bass => 25,
        }
    }

    /// Convert a MIDI pitch to a diatonic staff position.
    /// 0 = bottom line, 1 = bottom space, 2 = second line, ...
    /// Accidentals (sharps/flats) share the position of their natural note.
    pub const fn midi_to_staff_pos(self, pitch: MidiPitch) -> StaffPos {
        (midi_to_diatonic(pitch) - self.bottom_diatonic()) as StaffPos
    }

    /// Split point between treble and bass clefs: pitches >= this go to treble.
    /// Middle C (C4) goes to treble.
    pub const SPLIT_POINT: MidiPitch = 60;

    /// Choose a clef for a pitch, splitting at [`Clef::SPLIT_POINT`].
    pub const fn for_pitch(pitch: MidiPitch) -> Self {
        if pitch >= Self::SPLIT_POINT {
            Self::Treble
        } else {
            Self::Bass
        }
    }
}

/// Convert a MIDI pitch (0-127) to a diatonic index.
/// Diatonic index: 7 * octave + note_name_index
/// where note_name_index: C=0, D=1, E=2, F=3, G=4, A=5, B=6.
///
/// Chromatic notes (sharps/flats) share the diatonic index of their natural.
const fn midi_to_diatonic(pitch: MidiPitch) -> i16 {
    // Natural note index for each MIDI semitone 0-11
    const DIATONIC_INDEX: [i16; 12] = [
        0, // 0: C
        0, // 1: C# -> same position as C
        1, // 2: D
        1, // 3: D# -> same position as D
        2, // 4: E
        3, // 5: F
        3, // 6: F# -> same position as F
        4, // 7: G
        4, // 8: G# -> same position as G
        5, // 9: A
        5, // 10: A# -> same position as A
        6, // 11: B
    ];

    let semitone = (pitch % 12) as usize;
    let octave = (pitch / 12) as i16;
    octave * 7 + DIATONIC_INDEX[semitone]
}

/// Inclusive `(min, max)` staff position range for a single staff.
pub type StaffRange = (StaffPos, StaffPos);

/// A single staff line in the score: a flat, time-sorted list of elements
/// plus its vertical extent.
#[derive(Debug, Clone)]
pub struct Staff {
    pub elements: Vec<StaffElement>,
    /// `(min, max)` staff position observed across all notes on this staff.
    /// Used by the renderer to size top/bottom padding for ledger-line space.
    pub range: StaffRange,
}

/// The complete score generated from MIDI data.
#[derive(Debug, Clone)]
pub struct Score {
    pub treble: Staff,
    pub bass: Staff,
    /// Time at which the rendered score begins (typically `Duration::ZERO`).
    pub song_start: Duration,
    /// Time at which the rendered score ends.
    pub song_end: Duration,
}

// ── Conversion from MIDI ───────────────────────────────────────────────

/// Build a Score from MIDI file data.
///
/// Only notes from visible, non-drum tracks are included.
/// Notes are split between treble and bass staves at middle C (60).
pub fn score_from_midi(
    tracks: &[MidiTrack],
    hidden_tracks: &[usize],
    measures: &[Duration],
) -> Score {
    let notes = layout::collect_visible_notes(tracks, hidden_tracks);
    layout::build_score(&notes, measures)
}
