use std::time::Duration;

use midi_file::MidiTrack;

use crate::{
    Chord, Clef, MIDDLE_LINE_POS, MidiPitch, Notehead, Rest, RhythmicValue, Score, Staff,
    StaffElement, StaffPos, StaffRange, StemDirection,
};

/// A collected note with track metadata for the notation system.
#[derive(Debug, Clone)]
pub struct CollectedNote {
    pub start: Duration,
    pub duration: Duration,
    pub pitch: MidiPitch,
    pub color_id: usize,
}

/// Number of beats per measure assumed by the layout pipeline.
/// Time-signature support would replace this with per-measure data.
const BEATS_PER_MEASURE: u32 = 4;
/// Onset-time tolerance for grouping notes into a chord, in fractions of a beat.
const CHORD_GROUP_BEAT_FRACTION: u32 = 8;
/// Tolerance below which a sub-beat rest gap is treated as zero, in fractions of a beat.
const REST_EPS_BEAT_FRACTION: u32 = 32;

// ── Note collection ────────────────────────────────────────────────────

/// Collect all notes from visible tracks (excluding hidden and drum tracks).
pub fn collect_visible_notes(tracks: &[MidiTrack], hidden_tracks: &[usize]) -> Vec<CollectedNote> {
    let mut notes: Vec<CollectedNote> = tracks
        .iter()
        .filter(|track| {
            !hidden_tracks.contains(&track.track_id)
                && (!track.has_drums || track.has_other_than_drums)
        })
        .flat_map(|track| {
            track.notes.iter().map(move |note| CollectedNote {
                start: note.start,
                duration: note.duration,
                pitch: note.note,
                color_id: track.track_color_id,
            })
        })
        .collect();

    // Sort by start time, then by pitch (higher notes first for chord ordering)
    notes.sort_by(|a, b| a.start.cmp(&b.start).then_with(|| b.pitch.cmp(&a.pitch)));

    notes
}

// ── Duration quantization ──────────────────────────────────────────────

/// Rhythmically quantized note info.
#[derive(Debug, Clone)]
struct QuantizedNote {
    start: Duration,
    duration: Duration,
    pitch: MidiPitch,
    color_id: usize,
    rhythmic_value: RhythmicValue,
    dotted: bool,
}

/// `(rhythmic value, beat count)` candidates for quantization, longest first.
const QUANTIZE_CANDIDATES: [(RhythmicValue, f64); 6] = [
    (RhythmicValue::Whole, 4.0),
    (RhythmicValue::Half, 2.0),
    (RhythmicValue::Quarter, 1.0),
    (RhythmicValue::Eighth, 0.5),
    (RhythmicValue::Sixteenth, 0.25),
    (RhythmicValue::ThirtySecond, 0.125),
];

/// Quantize a MIDI duration to the nearest standard rhythmic value.
/// Returns (rhythmic_value, dotted).
fn quantize_duration(duration: Duration, beat_duration: Duration) -> (RhythmicValue, bool) {
    let beats = duration.as_secs_f64() / beat_duration.as_secs_f64();

    // Tolerance pass: snap to a candidate if within window. Dotted candidates
    // are tried first because at the same nominal value they have a wider
    // absolute window.
    for (rv, base) in &QUANTIZE_CANDIDATES {
        let dotted = base * 1.5;
        if (beats - dotted).abs() < dotted * 0.2 {
            return (*rv, true);
        }
        if (beats - base).abs() < base * 0.25 {
            return (*rv, false);
        }
    }

    // Fallback: find the closest match by absolute distance.
    let mut best = (RhythmicValue::Quarter, false);
    let mut best_diff = f64::MAX;
    for (rv, base) in &QUANTIZE_CANDIDATES {
        let diff = (beats - base).abs();
        if diff < best_diff {
            best_diff = diff;
            best = (*rv, false);
        }
        let dotted_diff = (beats - base * 1.5).abs();
        if dotted_diff < best_diff {
            best_diff = dotted_diff;
            best = (*rv, true);
        }
    }
    best
}

// ── Measure building ───────────────────────────────────────────────────

/// Default `(min, max)` staff range when no notes are present: the full
/// 5-line staff in diatonic-position units (0 = bottom line, 8 = top line).
const DEFAULT_RANGE: StaffRange = (0, 8);

/// Convert collected notes into a flat per-staff timeline, split by clef.
///
/// `measure_boundaries` is used only to derive `beat_duration` (for chord
/// grouping and rest quantization) and to bound the rendered timeline.
pub fn build_score(notes: &[CollectedNote], measure_boundaries: &[Duration]) -> Score {
    let song_start = measure_boundaries.first().copied().unwrap_or(Duration::ZERO);
    let song_end = measure_boundaries.last().copied().unwrap_or(Duration::ZERO);

    if measure_boundaries.len() < 2 {
        return Score {
            treble: Staff {
                elements: Vec::new(),
                range: DEFAULT_RANGE,
            },
            bass: Staff {
                elements: Vec::new(),
                range: DEFAULT_RANGE,
            },
            song_start,
            song_end,
        };
    }

    let measure_duration = measure_boundaries[1] - measure_boundaries[0];
    let beat_duration = measure_duration / BEATS_PER_MEASURE;

    // Quantize all notes and track each staff's vertical extent in one pass.
    // Notes are already sorted by start time (descending pitch on ties).
    let mut treble_range = DEFAULT_RANGE;
    let mut bass_range = DEFAULT_RANGE;
    let quantized: Vec<QuantizedNote> = notes
        .iter()
        .map(|n| {
            let clef = Clef::for_pitch(n.pitch);
            let pos = clef.midi_to_staff_pos(n.pitch);
            let range = match clef {
                Clef::Treble => &mut treble_range,
                Clef::Bass => &mut bass_range,
            };
            range.0 = range.0.min(pos);
            range.1 = range.1.max(pos);
            let (rv, dotted) = quantize_duration(n.duration, beat_duration);
            QuantizedNote {
                start: n.start,
                duration: n.duration,
                pitch: n.pitch,
                color_id: n.color_id,
                rhythmic_value: rv,
                dotted,
            }
        })
        .collect();

    // Group simultaneous notes (within 1/8 beat of the group's first onset)
    // into chords across the whole song.
    let chord_threshold = beat_duration / CHORD_GROUP_BEAT_FRACTION;
    let mut chord_groups: Vec<Vec<&QuantizedNote>> = Vec::new();
    for note in &quantized {
        let extends_last = chord_groups
            .last()
            .is_some_and(|last| note.start.saturating_sub(last[0].start) <= chord_threshold);
        if extends_last {
            chord_groups.last_mut().unwrap().push(note);
        } else {
            chord_groups.push(vec![note]);
        }
    }

    // For each chord group, emit one chord element per staff that has notes.
    let mut treble_chords: Vec<StaffElement> = Vec::new();
    let mut bass_chords: Vec<StaffElement> = Vec::new();
    for group in &chord_groups {
        // Groups are non-empty by construction, so first() is always Some.
        let group_start = group[0].start;
        let (treble_notes, bass_notes): (Vec<&QuantizedNote>, Vec<&QuantizedNote>) = group
            .iter()
            .copied()
            .partition(|n| Clef::for_pitch(n.pitch) == Clef::Treble);
        if !treble_notes.is_empty() {
            treble_chords.push(build_chord_element(Clef::Treble, &treble_notes, group_start));
        }
        if !bass_notes.is_empty() {
            bass_chords.push(build_chord_element(Clef::Bass, &bass_notes, group_start));
        }
    }

    let treble_elements = fill_rests(
        &treble_chords,
        song_start,
        song_end,
        beat_duration,
        measure_duration,
    );
    let bass_elements = fill_rests(
        &bass_chords,
        song_start,
        song_end,
        beat_duration,
        measure_duration,
    );

    Score {
        treble: Staff {
            elements: treble_elements,
            range: treble_range,
        },
        bass: Staff {
            elements: bass_elements,
            range: bass_range,
        },
        song_start,
        song_end,
    }
}

/// Build a single chord element for one staff from a subset of a chord group.
/// `notes` must be non-empty.
fn build_chord_element(
    clef: Clef,
    notes: &[&QuantizedNote],
    start_time: Duration,
) -> StaffElement {
    let heads: Vec<Notehead> = notes
        .iter()
        .map(|n| Notehead {
            staff_pos: clef.midi_to_staff_pos(n.pitch),
            color_id: n.color_id,
            start_time: n.start,
            end_time: n.start + n.duration,
        })
        .collect();

    // Stem direction: the head whose staff position is furthest from the
    // middle line wins, with the stem pointing AWAY from that side. Equal
    // distances point down (engraving convention).
    let (max_pos, min_pos) = heads.iter().map(|h| h.staff_pos).fold(
        (StaffPos::MIN, StaffPos::MAX),
        |(mx, mn), p| (mx.max(p), mn.min(p)),
    );
    let above = (max_pos as i16 - MIDDLE_LINE_POS as i16).max(0);
    let below = (MIDDLE_LINE_POS as i16 - min_pos as i16).max(0);
    let stem_direction = if above >= below {
        StemDirection::Down
    } else {
        StemDirection::Up
    };

    // Longest rhythmic value among heads on this staff drives the chord.
    let (rhythmic_value, dotted) = notes
        .iter()
        .map(|n| (n.rhythmic_value, n.dotted))
        .max_by(|a, b| a.0.beats_with_dot(a.1).total_cmp(&b.0.beats_with_dot(b.1)))
        .unwrap_or((RhythmicValue::Quarter, false));

    StaffElement::Chord(Chord {
        heads,
        rhythmic_value,
        dotted,
        stem_direction,
        start_time,
    })
}

/// Walk a staff's chord list in time order, inserting rests to fill silences.
/// `chords` contains only `StaffElement::Chord`, in time order. Long stretches
/// of silence emit one whole rest per `measure_duration` worth of time so the
/// eye still gets a "lots of rest here" signal.
fn fill_rests(
    chords: &[StaffElement],
    song_start: Duration,
    song_end: Duration,
    beat_duration: Duration,
    measure_duration: Duration,
) -> Vec<StaffElement> {
    let mut out: Vec<StaffElement> = Vec::with_capacity(chords.len());
    // Tolerance to avoid emitting tiny sub-millisecond rests from rounding.
    let eps = beat_duration / REST_EPS_BEAT_FRACTION;

    let mut cursor = song_start;
    for el in chords {
        let StaffElement::Chord(chord) = el else {
            continue;
        };
        push_silence_until(
            chord.start_time,
            &mut cursor,
            measure_duration,
            beat_duration,
            eps,
            &mut out,
        );
        out.push(el.clone());
        cursor =
            chord.start_time + chord.rhythmic_value.duration(chord.dotted, beat_duration);
    }

    // Trailing silence until song_end.
    push_silence_until(
        song_end,
        &mut cursor,
        measure_duration,
        beat_duration,
        eps,
        &mut out,
    );

    out
}

/// Emit rests from `*cursor` up to (but not past) `target`. Long gaps emit
/// one whole rest per `measure_duration`; the remainder is quantized.
fn push_silence_until(
    target: Duration,
    cursor: &mut Duration,
    measure_duration: Duration,
    beat_duration: Duration,
    eps: Duration,
    out: &mut Vec<StaffElement>,
) {
    while target >= *cursor + measure_duration + eps {
        out.push(StaffElement::Rest(Rest {
            rhythmic_value: RhythmicValue::Whole,
            dotted: false,
            start_time: *cursor,
        }));
        *cursor += measure_duration;
    }
    if target > *cursor + eps {
        let gap = target - *cursor;
        let (rv, dotted) = quantize_duration(gap, beat_duration);
        out.push(StaffElement::Rest(Rest {
            rhythmic_value: rv,
            dotted,
            start_time: *cursor,
        }));
        *cursor = target;
    }
}
