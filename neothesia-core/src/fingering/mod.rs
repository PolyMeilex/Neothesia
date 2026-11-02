//! Automatic piano fingering with user overrides

mod solver;
mod split;
mod store;

use midi_file::{MidiNote, MidiTrack};
use serde::{Deserialize, Serialize};

pub use store::{FingeringStore, NoteKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Left,
    Right,
}

impl Hand {
    /// Direction from the thumb towards the pinky, in pitch
    fn direction(self) -> i32 {
        match self {
            Hand::Left => -1,
            Hand::Right => 1,
        }
    }
}

/// Which hand plays a track
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HandAssignment {
    #[default]
    Auto,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteFingering {
    /// 1 (thumb) to 5 (pinky)
    pub finger: u8,
    pub hand: Hand,
    /// Set by the user rather than computed
    pub overridden: bool,
}

/// Resolved fingering of every note of a song
#[derive(Debug, Clone, Default)]
pub struct Fingering {
    /// Indexed by `[track_id][note.id]`
    tracks: Vec<Vec<Option<NoteFingering>>>,
}

impl Fingering {
    /// Computes the fingering of the given `(track_id, hand)` parts, other tracks are ignored
    pub fn compute(
        tracks: &[MidiTrack],
        parts: &[(usize, HandAssignment)],
        store: &FingeringStore,
    ) -> Self {
        let mut fingering = Self {
            tracks: tracks.iter().map(|t| vec![None; t.notes.len()]).collect(),
        };

        let part_notes = |track_id: usize| {
            tracks
                .iter()
                .find(|t| t.track_id == track_id)
                .into_iter()
                .flat_map(|t| t.notes.iter())
                .filter(|n| n.channel != 9)
        };

        let parts: Vec<(usize, HandAssignment)> = parts
            .iter()
            .copied()
            .filter(|(track_id, _)| part_notes(*track_id).next().is_some())
            .collect();

        let mut left: Vec<&MidiNote> = Vec::new();
        let mut right: Vec<&MidiNote> = Vec::new();
        let mut auto: Vec<&MidiNote> = Vec::new();

        let resolved = resolve_two_parts(&parts, |track_id| {
            let (sum, count) = part_notes(track_id).fold((0u64, 0u64), |(sum, count), n| {
                (sum + n.note as u64, count + 1)
            });
            sum as f32 / count as f32
        });

        for (track_id, assignment) in resolved {
            let target = match assignment {
                HandAssignment::Left => &mut left,
                HandAssignment::Right => &mut right,
                HandAssignment::Auto => &mut auto,
            };
            target.extend(part_notes(track_id));
        }

        if !auto.is_empty() {
            let input: Vec<_> = auto.iter().map(|n| (n.start, n.note)).collect();
            for (note, hand) in auto.iter().zip(split::split_hands(&input)) {
                match hand {
                    Hand::Left => left.push(note),
                    Hand::Right => right.push(note),
                }
            }
        }

        fingering.solve_hand(&left, Hand::Left, store);
        fingering.solve_hand(&right, Hand::Right, store);

        fingering
    }

    fn solve_hand(&mut self, notes: &[&MidiNote], hand: Hand, store: &FingeringStore) {
        let input: Vec<solver::SolverNote> = notes
            .iter()
            .map(|n| solver::SolverNote {
                start: n.start,
                end: n.end,
                pitch: n.note,
                fixed: store.overrides.get(&NoteKey::new(n)).copied(),
            })
            .collect();

        let out = solver::solve(&input, hand);

        for ((note, input), finger) in notes.iter().zip(&input).zip(out) {
            let Some(slot) = self
                .tracks
                .get_mut(note.track_id)
                .and_then(|t| t.get_mut(note.id))
            else {
                continue;
            };

            *slot = finger.map(|finger| NoteFingering {
                finger,
                hand,
                overridden: input.fixed.is_some(),
            });
        }
    }

    pub fn get(&self, note: &MidiNote) -> Option<NoteFingering> {
        self.tracks
            .get(note.track_id)?
            .get(note.id)
            .copied()
            .flatten()
    }
}

/// When a song has exactly two parts, they are most likely one per hand
fn resolve_two_parts(
    parts: &[(usize, HandAssignment)],
    mean_pitch: impl Fn(usize) -> f32,
) -> Vec<(usize, HandAssignment)> {
    use HandAssignment::{Auto, Left, Right};

    let [(a, hand_a), (b, hand_b)] = parts else {
        return parts.to_vec();
    };

    let (hand_a, hand_b) = match (*hand_a, *hand_b) {
        (Auto, Auto) => {
            if mean_pitch(*a) <= mean_pitch(*b) {
                (Left, Right)
            } else {
                (Right, Left)
            }
        }
        (Auto, Left) => (Right, Left),
        (Auto, Right) => (Left, Right),
        (Left, Auto) => (Left, Right),
        (Right, Auto) => (Right, Left),
        other => other,
    };

    vec![(*a, hand_a), (*b, hand_b)]
}
