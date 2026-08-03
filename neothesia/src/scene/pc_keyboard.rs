//! PC/HID keyboard -> MIDI note mapping for the configured piano range.
//!
//! The physical PC keyboard is arranged like a two-row mini piano:
//!
//! ```text
//!   w   e     t  y  u      o  p
//! a  s  d  f  g  h  j  k  l  ;  '
//! ```
//!
//! Each key is a chromatic step, so the whole thing spans 18 semitones
//! (one and a half octaves). The window is anchored at the C that sits
//! closest to the middle of the configured `piano_range`, so pressing a key
//! always lands on a visible, playable key. `x`/`z` shift that window up
//! and down by an octave to reach the rest of the range.

const PC_KEY_MAP: [char; 18] = [
    'a', 'w', 's', 'e', 'd', 'f', 't', 'g', 'y', 'h', 'u', 'j', 'k', 'o', 'l', 'p', ';', '\'',
];

pub fn key_index(c: char) -> Option<usize> {
    PC_KEY_MAP.iter().position(|&k| k == c)
}

/// Mutable state of the PC keyboard: how far the playable window is shifted
/// from its central position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PcKeyboard {
    octave_shift: i8,
}

impl PcKeyboard {
    pub fn shift_octave(&mut self, delta: i8) {
        self.octave_shift = self.octave_shift.saturating_add(delta);
    }

    /// Map a PC key index (`0..PC_KEY_MAP.len()`) to a MIDI note inside
    /// `range`, taking the configured piano range and the current octave
    /// shift into account.
    pub fn note_for(&self, index: usize, range: &std::ops::RangeInclusive<u8>) -> Option<u8> {
        if index >= PC_KEY_MAP.len() {
            return None;
        }

        let start = *range.start() as i16;
        let end = *range.end() as i16;

        // Anchor: the C to the left of the middle of the range.
        let mid = (start + end) / 2;
        let mut base = mid - mid.rem_euclid(12);

        // Keep the whole 18 semitone window inside the range when possible.
        let max_base = (end - PC_KEY_MAP.len() as i16 + 1).max(start);
        base = base.clamp(start, max_base);

        // Apply the user-controlled octave shift.
        base += self.octave_shift as i16 * 12;
        base = base.clamp(start, max_base);

        let note = (base + index as i16).clamp(start, end);

        Some(note as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn range(start: u8, end: u8) -> std::ops::RangeInclusive<u8> {
        start..=end
    }

    #[test]
    fn default_88_key_range_is_unchanged() {
        // 21..=108 keeps 'a' at C4 (60), same as before the change.
        let pc = PcKeyboard::default();
        let range = range(21, 108);

        assert_eq!(pc.note_for(0, &range), Some(60));
        assert_eq!(pc.note_for(17, &range), Some(77));
    }

    #[test]
    fn window_is_centered_in_the_range() {
        let pc = PcKeyboard::default();
        let range = range(43, 127);

        // Middle of the range is 85, closest C is C5 (84).
        assert_eq!(pc.note_for(0, &range), Some(84));
        assert_eq!(pc.note_for(17, &range), Some(101));
    }

    #[test]
    fn octave_shift_keeps_window_in_range() {
        let mut pc = PcKeyboard::default();
        let range = range(43, 127);

        pc.shift_octave(1);
        assert_eq!(pc.note_for(0, &range), Some(96));

        pc.shift_octave(-2);
        assert_eq!(pc.note_for(0, &range), Some(72));
    }

    #[test]
    fn octave_shift_clamps_at_the_edges() {
        let mut pc = PcKeyboard::default();
        let range = range(100, 127);

        pc.shift_octave(20);
        assert_eq!(pc.note_for(17, &range), Some(127));

        pc.shift_octave(-40);
        assert_eq!(pc.note_for(0, &range), Some(100));
    }

    #[test]
    fn small_range_collapses_notes_inside_it() {
        let pc = PcKeyboard::default();
        let range = range(60, 65);

        assert_eq!(pc.note_for(0, &range), Some(60));
        assert_eq!(pc.note_for(17, &range), Some(65));
    }

    #[test]
    fn out_of_index_returns_none() {
        let pc = PcKeyboard::default();
        assert_eq!(pc.note_for(PC_KEY_MAP.len(), &range(0, 127)), None);
    }
}
