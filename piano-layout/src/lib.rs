use std::rc::Rc;

pub mod range;
pub use range::KeyboardRange;

#[derive(Debug, Clone)]
pub struct KeyboardLayout {
    pub keys: Rc<[Key]>,

    pub width: f32,
    pub height: f32,

    pub sizing: Sizing,
    pub range: KeyboardRange,
}

impl KeyboardLayout {
    pub fn from_range(sizing: Sizing, range: KeyboardRange) -> Self {
        let mut keys = Vec::with_capacity(range.count());

        let mut offset = 0.0;
        let mut id = 0;

        let oct = Octave::new(&sizing);

        for octave_range in split_range_by_octaves(range.range()) {
            let (width, key_iter) = oct.sub_range(octave_range);

            for mut key in key_iter {
                key.id = id;
                id += 1;

                match key.kind {
                    KeyKind::Neutral => {
                        key.x += offset;
                    }
                    KeyKind::Sharp => {
                        key.x += offset;
                    }
                }

                keys.push(key);
            }

            offset += width;
        }

        // Board size
        let width = sizing.neutral_width * range.white_count() as f32;
        let height = sizing.neutral_height;

        KeyboardLayout {
            keys: keys.into(),

            width,
            height,

            sizing,
            range,
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyKind {
    #[default]
    Neutral,
    Sharp,
}

impl KeyKind {
    pub fn is_neutral(&self) -> bool {
        *self == Self::Neutral
    }

    pub fn is_sharp(&self) -> bool {
        *self == Self::Sharp
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Key {
    x: f32,
    width: f32,
    height: f32,
    kind: KeyKind,
    note_id: u8,
    id: usize,
}

impl Key {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn kind(&self) -> &KeyKind {
        &self.kind
    }

    pub fn note_id(&self) -> u8 {
        self.note_id
    }
}

fn split_range_by_octaves(
    range: &std::ops::Range<u8>,
) -> impl Iterator<Item = std::ops::Range<usize>> {
    let start = range.start as usize;
    let end = range.end as usize;

    let mut id = start;

    std::iter::from_fn(move || {
        if id < end {
            let start_id = id % 12;
            let end_id = if id + 12 > end { end - id } else { 12 };

            let range = start_id..end_id;

            id += range.len();

            Some(range)
        } else {
            None
        }
    })
}

struct Octave {
    keys: [Key; 12],
    width: f32,
}

impl Octave {
    fn new(sizing: &Sizing) -> Self {
        let mut keys = [Key::default(); 12];

        let width = sizing.neutral_width * 7.0;

        const C: u8 = 0;
        const CS: u8 = 1;
        const D: u8 = 2;
        const DS: u8 = 3;
        const E: u8 = 4;
        const F: u8 = 5;
        const FS: u8 = 6;
        const G: u8 = 7;
        const GS: u8 = 8;
        const A: u8 = 9;
        const AS: u8 = 10;
        const B: u8 = 11;

        let neutral_ids: [u8; 7] = [C, D, E, F, G, A, B];
        let sharp_ids: [u8; 5] = [CS, DS, FS, GS, AS];

        for (id, note_id) in neutral_ids.into_iter().enumerate() {
            let x = id as f32 * sizing.neutral_width;

            keys[note_id as usize] = Key {
                id: 0,
                x,
                width: sizing.neutral_width,
                height: sizing.neutral_height,
                kind: KeyKind::Neutral,
                note_id,
            };
        }

        #[inline(always)]
        fn sharp_note_id_to_x(note_id: u8, cde_width: f32, cde_mult: f32, fgab_mult: f32) -> f32 {
            if matches!(note_id, CS | DS) {
                let mult = cde_mult;
                (note_id + 1) as f32 * mult - mult / 2.0
            } else {
                let mult = fgab_mult;
                let id = note_id - E;
                cde_width + id as f32 * mult - mult / 2.0
            }
        }

        // Mathematically™ there is no correct way to position keys, but doing it separately for cde and fgh
        // is quite popular, and gives decently accurate results, so let's do that
        let cde_width = sizing.neutral_width * 3.0;
        let fgab_width = sizing.neutral_width * 4.0;
        let cde_mult = cde_width / 5.0;
        let fgab_mult = fgab_width / 7.0;

        for note_id in sharp_ids {
            let x = sharp_note_id_to_x(note_id, cde_width, cde_mult, fgab_mult);

            let w = sizing.sharp_width;
            let hw = w / 2.0;

            let x = x - hw;

            keys[note_id as usize] = Key {
                id: 0,
                x,
                width: sizing.sharp_width,
                height: sizing.sharp_height,
                kind: KeyKind::Sharp,
                note_id,
            };
        }

        Self { keys, width }
    }

    fn sub_range(&self, range: std::ops::Range<usize>) -> (f32, impl Iterator<Item = Key> + '_) {
        let keys = &self.keys[range.clone()];
        let start_offset = keys.first().map(Key::x).unwrap_or(0.0);
        let new_width = self.width - start_offset;

        let mut iter = keys.iter();

        (
            new_width,
            std::iter::from_fn(move || {
                let key = iter.next()?;

                Some(Key {
                    x: key.x - start_offset,
                    ..*key
                })
            }),
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sizing {
    pub neutral_width: f32,
    pub neutral_height: f32,

    pub sharp_width: f32,
    pub sharp_height: f32,
}

impl Sizing {
    pub fn new(neutral_width: f32, neutral_height: f32) -> Self {
        let sharp_width = neutral_width * 0.625; // 62.5%
        let sharp_height = neutral_height * 0.635;

        Self {
            neutral_width,
            neutral_height,
            sharp_width,
            sharp_height,
        }
    }
}
