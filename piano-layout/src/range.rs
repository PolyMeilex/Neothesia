use std::{
    ops::{Range, RangeBounds},
    rc::Rc,
};

const KEY_CIS: u8 = 1;
const KEY_DIS: u8 = 3;
const KEY_FIS: u8 = 6;
const KEY_GIS: u8 = 8;
const KEY_AIS: u8 = 10;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct KeyId(u8);

impl KeyId {
    pub fn is_black(&self) -> bool {
        let key = self.0 % 12;
        key == KEY_CIS || key == KEY_DIS || key == KEY_FIS || key == KEY_GIS || key == KEY_AIS
    }
}

/// Describe used slice of piano keyboard
#[derive(Debug, Clone)]
pub struct KeyboardRange {
    range: Range<u8>,

    keys: Rc<[KeyId]>,
    white_keys: Rc<[KeyId]>,
    black_keys: Rc<[KeyId]>,
}

impl KeyboardRange {
    pub fn new<R>(range: R) -> Self
    where
        R: RangeBounds<u8>,
    {
        let mut keys = Vec::new();
        let mut white_keys = Vec::new();
        let mut black_keys = Vec::new();

        let start = range.start_bound();
        let end = range.end_bound();

        let start = match start {
            std::ops::Bound::Included(id) => *id,
            std::ops::Bound::Excluded(id) => *id + 1,
            std::ops::Bound::Unbounded => 0,
        };

        let end = match end {
            std::ops::Bound::Included(id) => *id + 1,
            std::ops::Bound::Excluded(id) => *id,
            std::ops::Bound::Unbounded => 128,
        };

        // 127 is the top of MIDI tuning range
        let range = start..end.min(128);

        for id in range.clone().map(KeyId) {
            keys.push(id);

            if id.is_black() {
                black_keys.push(id);
            } else {
                white_keys.push(id);
            }
        }

        Self {
            range,

            keys: keys.into(),
            white_keys: white_keys.into(),
            black_keys: black_keys.into(),
        }
    }

    pub fn standard_88_keys() -> Self {
        Self::new(21..=108)
    }

    pub fn start(&self) -> u8 {
        self.range.start
    }

    pub fn end(&self) -> u8 {
        self.range.end
    }
}

impl KeyboardRange {
    pub fn range(&self) -> &std::ops::Range<u8> {
        &self.range
    }

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

    pub fn iter(&self) -> std::slice::Iter<'_, KeyId> {
        self.keys.iter()
    }

    pub fn white_iter(&self) -> std::slice::Iter<'_, KeyId> {
        self.white_keys.iter()
    }

    pub fn black_iter(&self) -> std::slice::Iter<'_, KeyId> {
        self.black_keys.iter()
    }
}

impl Default for KeyboardRange {
    fn default() -> Self {
        Self::standard_88_keys()
    }
}
