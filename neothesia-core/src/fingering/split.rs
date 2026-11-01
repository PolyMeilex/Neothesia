//! Splits notes of a single piano part between the left and the right hand.

use std::time::Duration;

use super::Hand;

const CHORD_TOLERANCE: Duration = Duration::from_millis(30);
/// Widest comfortable chord of a single hand, in semitones
const MAX_HAND_SPAN: i32 = 14;

/// Returns a hand for every `(start, pitch)` input note, in input order
pub fn split_hands(notes: &[(Duration, u8)]) -> Vec<Hand> {
    let mut order: Vec<usize> = (0..notes.len()).collect();
    order.sort_by_key(|&id| (notes[id].0, notes[id].1));

    let mut out = vec![Hand::Right; notes.len()];

    // Running estimate of where each hand currently is
    let mut left_center = 48.0f32;
    let mut right_center = 72.0f32;

    let mut id = 0;
    while id < order.len() {
        let start = notes[order[id]].0;
        let mut members = Vec::new();
        while id < order.len() && notes[order[id]].0 - start <= CHORD_TOLERANCE {
            members.push(order[id]);
            id += 1;
        }

        // Members are sorted by pitch, pick the best split point:
        // `members[..split]` go to the left hand, the rest to the right hand
        let pitches: Vec<i32> = members.iter().map(|&m| notes[m].1 as i32).collect();

        let split = (0..=pitches.len())
            .map(|split| {
                let (left, right) = pitches.split_at(split);
                let cost = hand_cost(left, left_center) + hand_cost(right, right_center);
                (split, cost)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(split, _)| split)
            .unwrap();

        let (left, right) = pitches.split_at(split);
        if !left.is_empty() {
            left_center = 0.7 * left_center + 0.3 * mean(left);
        }
        if !right.is_empty() {
            right_center = 0.7 * right_center + 0.3 * mean(right);
        }

        // Keep hands from collapsing onto each other
        if right_center - left_center < 7.0 {
            let mid = (left_center + right_center) / 2.0;
            left_center = mid - 3.5;
            right_center = mid + 3.5;
        }

        for (pos, &m) in members.iter().enumerate() {
            out[m] = if pos < split { Hand::Left } else { Hand::Right };
        }
    }

    out
}

fn hand_cost(pitches: &[i32], center: f32) -> f32 {
    let (Some(first), Some(last)) = (pitches.first(), pitches.last()) else {
        return 0.0;
    };

    let mut cost: f32 = pitches.iter().map(|&p| (p as f32 - center).abs()).sum();

    let span = last - first;
    if span > MAX_HAND_SPAN {
        cost += 100.0 * (span - MAX_HAND_SPAN) as f32;
    }
    if pitches.len() > 5 {
        cost += 100.0 * (pitches.len() - 5) as f32;
    }

    cost
}

fn mean(pitches: &[i32]) -> f32 {
    pitches.iter().sum::<i32>() as f32 / pitches.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bass_and_melody() {
        let notes = [
            (Duration::ZERO, 36),
            (Duration::ZERO, 72),
            (Duration::from_millis(500), 43),
            (Duration::from_millis(500), 76),
        ];
        assert_eq!(
            split_hands(&notes),
            [Hand::Left, Hand::Right, Hand::Left, Hand::Right]
        );
    }

    #[test]
    fn wide_chord_is_split() {
        let notes = [
            (Duration::ZERO, 48),
            (Duration::ZERO, 55),
            (Duration::ZERO, 64),
            (Duration::ZERO, 67),
            (Duration::ZERO, 72),
        ];
        let hands = split_hands(&notes);
        assert_eq!(hands[0], Hand::Left);
        assert_eq!(hands[4], Hand::Right);
    }
}
