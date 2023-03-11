pub mod range;
pub use range::KeyboardRange;

#[derive(Debug, Clone)]
pub struct KeyboardLayout {
    pub keys: Vec<Key>,

    pub width: f32,
    pub height: f32,

    pub neutral_width: f32,
    pub sharp_width: f32,

    pub neutral_height: f32,
    pub sharp_height: f32,

    pub range: KeyboardRange,
}

#[derive(Debug, Clone)]
pub enum KeyKind {
    Neutral,
    Sharp,
}

impl KeyKind {
    pub fn is_neutral(&self) -> bool {
        matches!(self, Self::Neutral)
    }

    pub fn is_sharp(&self) -> bool {
        matches!(self, Self::Sharp)
    }
}

#[derive(Debug, Clone)]
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

struct Octave {
    keys: Vec<Key>,
    width: f32,
}

struct Sizing {
    neutral_width: f32,
    neutral_height: f32,

    sharp_width: f32,
    sharp_height: f32,
}

pub fn standard_88_keys(neutral_width: f32, neutral_height: f32) -> KeyboardLayout {
    let sharp_width = neutral_width * 0.625; // 62.5%
    let sharp_height = neutral_height * 0.635;

    let sizing = Sizing {
        neutral_width,
        neutral_height,
        sharp_width,
        sharp_height,
    };

    let octaves = [
        partial_octave(&sizing, 9..12),
        octave(&sizing),
        octave(&sizing),
        octave(&sizing),
        octave(&sizing),
        octave(&sizing),
        octave(&sizing),
        octave(&sizing),
        partial_octave(&sizing, 0..1),
    ];

    let mut keys = Vec::with_capacity(88);

    let mut offset = 0.0;
    let mut id = 0;

    for octave in octaves {
        for mut key in octave.keys {
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

        offset += octave.width;
    }

    // Board size
    let width = neutral_width * 52.0; // Neutral keys count
    let height = neutral_height;

    KeyboardLayout {
        keys,

        width,
        height,

        neutral_width,
        sharp_width,

        neutral_height,
        sharp_height,

        range: KeyboardRange::standard_88_keys(),
    }
}

fn octave(sizing: &Sizing) -> Octave {
    partial_octave(sizing, 0..12)
}

fn partial_octave(sizing: &Sizing, range: std::ops::Range<u8>) -> Octave {
    let mut keys = Vec::with_capacity(12);

    let width = sizing.neutral_width * 7.0;

    let neutral_ids: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

    for (id, note_id) in neutral_ids.iter().enumerate() {
        let x = id as f32 * sizing.neutral_width;

        if range.contains(note_id) {
            keys.push(Key {
                id: 0,
                x,
                width: sizing.neutral_width,
                height: sizing.neutral_height,
                kind: KeyKind::Neutral,
                note_id: *note_id,
            });
        }
    }

    let sharp_ids: [u8; 5] = [1, 3, 6, 8, 10];

    #[inline(always)]
    fn sharp_id_to_x(id: u8, mult: f32) -> f32 {
        (id + 1) as f32 * mult
    }

    let mult = width / 12.0;
    let last_x = sharp_id_to_x(sharp_ids[4], mult);
    let offset = (width - last_x) / 2.0;

    for note_id in sharp_ids {
        let x = sharp_id_to_x(note_id, mult);
        let x = x - offset;

        let w = sizing.sharp_width;
        let hw = w / 2.0;

        let x = x - hw;

        if range.contains(&note_id) {
            keys.push(Key {
                id: 0,
                x,
                width: sizing.sharp_width,
                height: sizing.sharp_height,
                kind: KeyKind::Sharp,
                note_id,
            });
        }
    }

    let start_offset = keys.first().map(|key| key.x());

    if let Some(start_offset) = start_offset {
        keys.iter_mut().for_each(|key| key.x -= start_offset);
    }

    keys.sort_by_key(|key| key.note_id);

    Octave {
        keys,
        width: width - start_offset.unwrap_or(0.0),
    }
}
