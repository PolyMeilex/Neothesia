//! Piano fingering optimizer.
//!
//! Notes of a single hand are grouped into slices of (nearly) simultaneous onsets, every slice
//! gets a set of candidate fingerings and a Viterbi pass picks the cheapest path through them.
//! The cost model is loosely based on Parncutt et al. (1997) "An ergonomic model of keyboard
//! fingering for melodic fragments", extended with a vertical cost for chords.

use std::time::Duration;

use super::Hand;

/// Notes starting within this window are treated as a chord
const CHORD_TOLERANCE: Duration = Duration::from_millis(30);

#[derive(Debug, Clone)]
pub struct SolverNote {
    pub start: Duration,
    pub end: Duration,
    pub pitch: u8,
    /// Finger forced by the user
    pub fixed: Option<u8>,
}

/// Returns a finger (1-5) for every input note, in input order.
///
/// Notes that can not be fingered (more than 5 distinct keys at once) get `None`.
pub fn solve(notes: &[SolverNote], hand: Hand) -> Vec<Option<u8>> {
    let dir = hand.direction();

    let mut order: Vec<usize> = (0..notes.len()).collect();
    order.sort_by_key(|&id| (notes[id].start, notes[id].pitch as i32 * dir));

    let slices = build_slices(notes, &order, dir);
    let mut out = vec![None; notes.len()];
    if slices.is_empty() {
        return out;
    }

    // Viterbi
    let mut costs: Vec<Vec<f32>> = Vec::with_capacity(slices.len());
    let mut back: Vec<Vec<usize>> = Vec::with_capacity(slices.len());

    for (slice_id, slice) in slices.iter().enumerate() {
        let state_costs: Vec<f32> = slice.states.iter().map(|s| slice.state_cost(s)).collect();

        if slice_id == 0 {
            let state_costs = slice
                .states
                .iter()
                .zip(state_costs)
                .map(|(state, cost)| cost + state.iter().map(|&f| weak_finger_cost(f)).sum::<f32>())
                .collect();
            costs.push(state_costs);
            back.push(vec![0; slice.states.len()]);
            continue;
        }

        let prev = &slices[slice_id - 1];
        let prev_costs = &costs[slice_id - 1];
        let gap = slice.start.saturating_sub(prev.end);

        let mut slice_costs = Vec::with_capacity(slice.states.len());
        let mut slice_back = Vec::with_capacity(slice.states.len());

        for (state, state_cost) in slice.states.iter().zip(state_costs) {
            let (best_id, best_cost) = prev
                .states
                .iter()
                .zip(prev_costs)
                .map(|(prev_state, prev_cost)| {
                    prev_cost + transition_cost(prev, prev_state, slice, state, gap, dir)
                })
                .enumerate()
                .min_by(|a, b| a.1.total_cmp(&b.1))
                .unwrap();

            slice_costs.push(best_cost + state_cost);
            slice_back.push(best_id);
        }

        costs.push(slice_costs);
        back.push(slice_back);
    }

    let mut state_id = costs
        .last()
        .unwrap()
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.total_cmp(b.1))
        .map(|(id, _)| id)
        .unwrap();

    for slice_id in (0..slices.len()).rev() {
        let slice = &slices[slice_id];
        let state = &slice.states[state_id];

        for &(note_id, key_id) in &slice.notes {
            out[note_id] = key_id.map(|key_id| state[key_id]);
        }

        state_id = back[slice_id][state_id];
    }

    // User choice always wins, even when the constraint could not be satisfied
    for (out, note) in out.iter_mut().zip(notes) {
        if let Some(fixed) = note.fixed {
            *out = Some(fixed);
        }
    }

    out
}

struct Slice {
    start: Duration,
    end: Duration,
    /// Distinct keys, ordered from the thumb side to the pinky side
    keys: Vec<u8>,
    /// `(note id, key id)`
    notes: Vec<(usize, Option<usize>)>,
    /// Candidate fingerings, one finger per key
    states: Vec<Vec<u8>>,
}

impl Slice {
    fn state_cost(&self, state: &[u8]) -> f32 {
        let mut cost = 0.0;

        for (&key, &finger) in self.keys.iter().zip(state) {
            cost += finger_cost(finger, key);
        }

        // Vertical stretch inside of a chord
        for id in 1..self.keys.len() {
            let span = (self.keys[id] as i32 - self.keys[id - 1] as i32).abs();
            cost += span_cost(state[id - 1], state[id], span, 1.0);
        }

        cost
    }
}

fn build_slices(notes: &[SolverNote], order: &[usize], dir: i32) -> Vec<Slice> {
    let mut slices: Vec<Slice> = Vec::new();

    let mut id = 0;
    while id < order.len() {
        let start = notes[order[id]].start;
        let mut members = Vec::new();
        while id < order.len() && notes[order[id]].start - start <= CHORD_TOLERANCE {
            members.push(order[id]);
            id += 1;
        }

        let end = members.iter().map(|&m| notes[m].end).max().unwrap();

        let mut keys: Vec<u8> = members.iter().map(|&m| notes[m].pitch).collect();
        keys.sort_by_key(|&p| p as i32 * dir);
        keys.dedup();

        // A hand has only 5 fingers, keep an evenly spread subset of the keys
        if keys.len() > 5 {
            let n = keys.len() - 1;
            keys = (0..5).map(|i| keys[(i * n + 2) / 4]).collect();
            keys.dedup();
        }

        let notes_map: Vec<(usize, Option<usize>)> = members
            .iter()
            .map(|&m| (m, keys.iter().position(|&k| k == notes[m].pitch)))
            .collect();

        let mut fixed: Vec<Option<u8>> = vec![None; keys.len()];
        for &(note_id, key_id) in &notes_map {
            if let (Some(key_id), Some(finger)) = (key_id, notes[note_id].fixed) {
                fixed[key_id] = Some(finger);
            }
        }

        let all_states = finger_combinations(keys.len());
        let constrained: Vec<Vec<u8>> = all_states
            .iter()
            .filter(|state| {
                state
                    .iter()
                    .zip(&fixed)
                    .all(|(finger, fixed)| fixed.is_none_or(|f| f == *finger))
            })
            .cloned()
            .collect();

        let states = if constrained.is_empty() {
            all_states
        } else {
            constrained
        };

        slices.push(Slice {
            start,
            end,
            keys,
            notes: notes_map,
            states,
        });
    }

    slices
}

/// All strictly increasing finger sequences of length `len`
fn finger_combinations(len: usize) -> Vec<Vec<u8>> {
    fn rec(from: u8, len: usize, current: &mut Vec<u8>, out: &mut Vec<Vec<u8>>) {
        if current.len() == len {
            out.push(current.clone());
            return;
        }
        for finger in from..=5 {
            current.push(finger);
            rec(finger + 1, len, current, out);
            current.pop();
        }
    }

    let mut out = Vec::new();
    rec(1, len, &mut Vec::with_capacity(len), &mut out);
    out
}

fn transition_cost(
    prev: &Slice,
    prev_state: &[u8],
    next: &Slice,
    next_state: &[u8],
    gap: Duration,
    dir: i32,
) -> f32 {
    // The hand can freely reposition during a rest
    let legato = {
        let gap = gap.as_secs_f32();
        (1.0 - (gap - 0.1) / 0.4).clamp(0.2, 1.0)
    };

    let mut cost = 0.0;

    for (&next_key, &next_finger) in next.keys.iter().zip(next_state) {
        // Compare with the finger of the previous slice that is the closest to this one
        let (&prev_key, &prev_finger) = prev
            .keys
            .iter()
            .zip(prev_state)
            .min_by_key(|(key, finger)| {
                (
                    (**finger as i32 - next_finger as i32).abs(),
                    (**key as i32 - next_key as i32).abs(),
                )
            })
            .unwrap();

        cost += pair_cost(prev_finger, prev_key, next_finger, next_key, dir, legato);

        if (prev_key, prev_finger) != (next_key, next_finger) {
            cost += weak_finger_cost(next_finger);
        }
    }

    cost / next.keys.len() as f32
}

/// Cost of playing `key_b` with `finger_b` right after `key_a` with `finger_a`
fn pair_cost(finger_a: u8, key_a: u8, finger_b: u8, key_b: u8, dir: i32, legato: f32) -> f32 {
    let distance = (key_b as i32 - key_a as i32) * dir;
    let shift = hand_shift_cost(finger_a, key_a, finger_b, key_b, dir, legato);

    if key_a == key_b {
        // Finger substitution on a repeated key
        return if finger_a == finger_b {
            0.0
        } else {
            1.5 + shift
        };
    }

    if finger_a == finger_b {
        return (5.0 + 0.5 * distance.abs() as f32) * legato + shift;
    }

    // Span between the lower and the higher finger, negative when crossing
    let (lo, hi, span) = if finger_a < finger_b {
        (finger_a, finger_b, distance)
    } else {
        (finger_b, finger_a, -distance)
    };

    let mut cost = span_cost(lo, hi, span, legato) + shift;

    // Three-to-four rule
    if finger_a == 3 && finger_b == 4 {
        cost += 1.0;
    }

    // Four-on-black rule
    let (key_3, key_4) = match (finger_a, finger_b) {
        (3, 4) => (Some(key_a), Some(key_b)),
        (4, 3) => (Some(key_b), Some(key_a)),
        _ => (None, None),
    };
    if let (Some(key_3), Some(key_4)) = (key_3, key_4)
        && !is_black(key_3)
        && is_black(key_4)
    {
        cost += 1.0;
    }

    // Thumb-passing rule, with a base cost per hand position change so that
    // fewer, longer positions are preferred over frequent short ones
    if lo == 1 && span < 0 {
        let thumb_key = if finger_a == 1 { key_a } else { key_b };
        let other_key = if finger_a == 1 { key_b } else { key_a };

        cost += 3.0;
        if is_black(thumb_key) == is_black(other_key) {
            cost += 1.0;
        } else if is_black(thumb_key) {
            cost += 3.0;
        }
    }

    cost
}

/// Cost of moving the whole hand to a new position.
///
/// Every finger implies where the thumb is, assuming a relaxed five-finger position
/// on neighbouring white keys. Staying in one position is preferred, which keeps
/// eg. a C-F-G bass on 5-2-1 instead of jumping between arbitrary fingers.
fn hand_shift_cost(finger_a: u8, key_a: u8, finger_b: u8, key_b: u8, dir: i32, legato: f32) -> f32 {
    const OFFSET: [i32; 5] = [0, 2, 4, 5, 7];

    let thumb_a = key_a as i32 * dir - OFFSET[finger_a as usize - 1];
    let thumb_b = key_b as i32 * dir - OFFSET[finger_b as usize - 1];

    // Repositioning during a rest is cheaper, but never free
    0.4 * (thumb_b - thumb_a).abs() as f32 * legato.max(0.5)
}

/// Stretch, small-span and large-span rules for a pair of fingers, `lo < hi`
fn span_cost(lo: u8, hi: u8, span: i32, weight: f32) -> f32 {
    let s = spans(lo, hi);
    let mut cost = 0.0;

    // Stretch rule
    if span < s.min_comf {
        cost += 2.0 * (s.min_comf - span) as f32;
    } else if span > s.max_comf {
        cost += 2.0 * (span - s.max_comf) as f32;
    }

    // Practically impossible
    if span < s.min_prac {
        cost += 10.0 * (s.min_prac - span) as f32;
    } else if span > s.max_prac {
        cost += 10.0 * (span - s.max_prac) as f32;
    }

    // Small-span rule, thumb crossings are the main contributor here
    if span < s.min_rel {
        let per_semitone = if lo == 1 { 2.0 } else { 1.0 };
        cost += (s.min_rel - span) as f32 * per_semitone;
    }

    // Large-span rule
    if span > s.max_rel {
        let per_semitone = if lo == 1 { 1.0 } else { 2.0 };
        cost += (span - s.max_rel) as f32 * per_semitone;
    }

    cost * weight
}

/// Weak-finger rule, only paid when the finger strikes a new key, so that
/// repeated notes do not push the hand out of its position
fn weak_finger_cost(finger: u8) -> f32 {
    match finger {
        4 => 0.5,
        5 => 0.25,
        _ => 0.0,
    }
}

/// Static cost of a single finger on a key
fn finger_cost(finger: u8, key: u8) -> f32 {
    let mut cost = 0.0;

    if is_black(key) {
        cost += match finger {
            1 => 1.0,
            5 => 1.0,
            _ => 0.0,
        };
    }

    cost
}

fn is_black(key: u8) -> bool {
    matches!(key % 12, 1 | 3 | 6 | 8 | 10)
}

/// Spans in semitones, right hand, from the thumb side
struct Spans {
    min_prac: i32,
    min_comf: i32,
    min_rel: i32,
    max_rel: i32,
    max_comf: i32,
    max_prac: i32,
}

const fn spans(lo: u8, hi: u8) -> Spans {
    let [min_prac, min_comf, min_rel, max_rel, max_comf, max_prac] = match (lo, hi) {
        (1, 2) => [-5, -3, 1, 5, 8, 10],
        (1, 3) => [-4, -2, 3, 7, 10, 12],
        (1, 4) => [-3, -1, 5, 9, 12, 14],
        (1, 5) => [-1, 1, 7, 10, 13, 15],
        (2, 3) => [1, 1, 1, 2, 3, 5],
        (2, 4) => [1, 1, 3, 4, 5, 7],
        (2, 5) => [2, 2, 5, 6, 8, 10],
        (3, 4) => [1, 1, 1, 2, 2, 4],
        (3, 5) => [1, 1, 3, 4, 5, 7],
        (4, 5) => [1, 1, 1, 2, 3, 5],
        _ => [0, 0, 0, 0, 0, 0],
    };

    Spans {
        min_prac,
        min_comf,
        min_rel,
        max_rel,
        max_comf,
        max_prac,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn melody(pitches: &[u8]) -> Vec<SolverNote> {
        pitches
            .iter()
            .enumerate()
            .map(|(id, &pitch)| SolverNote {
                start: Duration::from_millis(id as u64 * 250),
                end: Duration::from_millis(id as u64 * 250 + 240),
                pitch,
                fixed: None,
            })
            .collect()
    }

    fn fingers(out: Vec<Option<u8>>) -> Vec<u8> {
        out.into_iter().map(Option::unwrap).collect()
    }

    const C_MAJOR: [u8; 8] = [60, 62, 64, 65, 67, 69, 71, 72];

    #[test]
    fn c_major_scale_right_hand() {
        let out = solve(&melody(&C_MAJOR), Hand::Right);
        assert_eq!(fingers(out), [1, 2, 3, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn c_major_scale_left_hand() {
        let scale: Vec<u8> = C_MAJOR.iter().map(|p| p - 12).collect();
        let out = solve(&melody(&scale), Hand::Left);
        assert_eq!(fingers(out), [5, 4, 3, 2, 1, 3, 2, 1]);
    }

    #[test]
    fn c_major_scale_descending_right_hand() {
        let scale: Vec<u8> = C_MAJOR.iter().rev().copied().collect();
        let out = solve(&melody(&scale), Hand::Right);
        assert_eq!(fingers(out), [5, 4, 3, 2, 1, 3, 2, 1]);
    }

    #[test]
    fn g_major_scale_right_hand() {
        let out = solve(&melody(&[67, 69, 71, 72, 74, 76, 78, 79]), Hand::Right);
        assert_eq!(fingers(out), [1, 2, 3, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn two_octave_scale_right_hand() {
        let scale = [60, 62, 64, 65, 67, 69, 71, 72, 74, 76, 77, 79, 81, 83, 84];
        let out = solve(&melody(&scale), Hand::Right);
        assert_eq!(fingers(out), [1, 2, 3, 1, 2, 3, 4, 1, 2, 3, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn alle_meine_entchen() {
        // C D E F G G | A A A A G | A A A A G | F F F F E E | D D D D C
        let melody_keys = [
            60, 62, 64, 65, 67, 67, 69, 69, 69, 69, 67, 69, 69, 69, 69, 67, 65, 65, 65, 65, 64, 64,
            62, 62, 62, 62, 60,
        ];
        let out = fingers(solve(&melody(&melody_keys), Hand::Right));
        assert_eq!(
            out,
            [
                1, 2, 3, 4, 5, 4, 5, 5, 5, 5, 4, 5, 5, 5, 5, 4, 3, 3, 3, 4, 3, 3, 2, 2, 2, 2, 1
            ]
        );

        // C-F-G accompaniment stays in one hand position
        let bass = fingers(solve(&melody(&[48, 53, 48, 53, 48, 55, 48]), Hand::Left));
        assert_eq!(bass, [5, 2, 5, 2, 5, 1, 5]);
    }

    #[test]
    fn triad() {
        let chord: Vec<SolverNote> = [60, 64, 67]
            .iter()
            .map(|&pitch| SolverNote {
                start: Duration::ZERO,
                end: Duration::from_millis(500),
                pitch,
                fixed: None,
            })
            .collect();

        assert_eq!(fingers(solve(&chord, Hand::Right)), [1, 3, 5]);
        assert_eq!(fingers(solve(&chord, Hand::Left)), [5, 3, 1]);
    }

    #[test]
    fn fixed_finger_is_respected() {
        let mut notes = melody(&C_MAJOR);
        notes[3].fixed = Some(4);
        let out = fingers(solve(&notes, Hand::Right));
        assert_eq!(out[3], 4);
        // Neighbours adapt to the forced finger
        assert_eq!(&out[..3], [1, 2, 3]);
    }

    #[test]
    fn too_many_keys() {
        let chord: Vec<SolverNote> = (60..67)
            .map(|pitch| SolverNote {
                start: Duration::ZERO,
                end: Duration::from_millis(500),
                pitch,
                fixed: None,
            })
            .collect();

        let out = solve(&chord, Hand::Right);
        assert_eq!(out.iter().filter(|f| f.is_some()).count(), 5);
    }
}
