#![allow(dead_code)]

use std::ops::RangeInclusive;

const KEY_CIS: u8 = 1;
const KEY_DIS: u8 = 3;
const KEY_FIS: u8 = 6;
const KEY_GIS: u8 = 8;
const KEY_AIS: u8 = 10;
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct KeyId(u8);

impl KeyId {
    pub fn is_black(&self) -> bool {
        let key = self.0 % 12;
        key == KEY_CIS || key == KEY_DIS || key == KEY_FIS || key == KEY_GIS || key == KEY_AIS
    }
}

/// Describe used slice of piano keyboard
pub struct KeyboardRange {
    range: RangeInclusive<u8>,

    keys: Vec<KeyId>,
    white_keys: Vec<KeyId>,
    black_keys: Vec<KeyId>,
}

impl KeyboardRange {
    pub fn new(range: RangeInclusive<u8>) -> Self {
        let mut keys = Vec::new();
        let mut white_keys = Vec::new();
        let mut black_keys = Vec::new();

        for id in range.clone().map(KeyId) {
            keys.push(id);

            if id.is_black() {
                black_keys.push(id);
            } else {
                white_keys.push(id);
            }
        }

        assert_eq!(white_keys.len(), 52);
        assert_eq!(black_keys.len(), 36);
        assert_eq!(white_keys.len() + black_keys.len(), 88);

        Self {
            range,

            keys,
            white_keys,
            black_keys,
        }
    }

    pub fn standard_88_keys() -> Self {
        Self::new(21..=108)
    }
}

impl KeyboardRange {
    pub fn contains(&self, item: u8) -> bool {
        self.range.contains(&item)
    }

    pub fn count(&self) -> usize {
        self.keys.len()
    }

    pub fn white_count(&self) -> usize {
        self.white_keys.len()
    }

    pub fn black_count(&self) -> usize {
        self.black_keys.len()
    }

    pub fn iter(&self) -> std::slice::Iter<KeyId> {
        self.keys.iter()
    }

    pub fn white_iter(&self) -> std::slice::Iter<KeyId> {
        self.white_keys.iter()
    }

    pub fn black_iter(&self) -> std::slice::Iter<KeyId> {
        self.black_keys.iter()
    }
}

impl Default for KeyboardRange {
    fn default() -> Self {
        Self::standard_88_keys()
    }
}

#[test]
fn range() {
    KeyboardRange::new(21..=108);
}
