use crate::{
    config::NoteNaming,
    render::{QuadInstance, QuadRenderer},
    utils::Point,
};

use piano_layout::range::KeyboardRange;

mod key_state;
pub use key_state::KeyState;

use super::{
    TextRenderer,
    note_labels::SHARP_NOTES,
    text::{dark_text_on, label_shaping, text_color_on},
};

const MIDDLE_C: u8 = 60;
const NEUTRAL_NOTES: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

#[derive(Clone, Copy)]
struct MiddleCDot {
    key_id: usize,
    x: f32,
    y: f32,
    size: f32,
}

struct NameSizes {
    neutral: f32,
    sharp: f32,
    accidental_below: bool,
}

pub struct KeyboardRenderer {
    pos: Point<f32>,

    key_states: Vec<KeyState>,

    layout: piano_layout::KeyboardLayout,
    note_naming: NoteNaming,
    key_labels: bool,

    cache: Vec<QuadInstance>,
    text_cache: Vec<(usize, super::text::TextArea)>,
    middle_c_dot: Option<MiddleCDot>,
}

impl KeyboardRenderer {
    pub fn new(layout: piano_layout::KeyboardLayout) -> Self {
        let key_states: Vec<KeyState> = layout
            .range
            .iter()
            .map(|id| KeyState::new(id.is_black()))
            .collect();

        let cache = Vec::with_capacity(key_states.len() + 3);

        Self {
            pos: Default::default(),

            key_states,

            layout,
            note_naming: NoteNaming::default(),
            key_labels: false,
            cache,
            text_cache: Vec::new(),
            middle_c_dot: None,
        }
    }

    pub fn reset_notes(&mut self) {
        for key in self.key_states.iter_mut() {
            key.pressed_by_file_off();
        }
        self.invalidate_cache();
    }

    pub fn range(&self) -> &KeyboardRange {
        &self.layout.range
    }

    pub fn key_states(&self) -> &[KeyState] {
        &self.key_states
    }

    pub fn key_states_mut(&mut self) -> &mut [KeyState] {
        &mut self.key_states
    }

    pub fn pos(&self) -> &Point<f32> {
        &self.pos
    }

    pub fn position_on_bottom_of_parent(&mut self, parent_height: f32) {
        let h = self.layout.height;
        let y = parent_height - h;

        self.set_pos((0.0, y).into());
    }

    pub fn set_pos(&mut self, pos: Point<f32>) {
        self.pos = pos;
        self.invalidate_layout_cache();
    }

    pub fn layout(&self) -> &piano_layout::KeyboardLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: piano_layout::KeyboardLayout) {
        self.layout = layout;
        self.invalidate_layout_cache();
    }

    pub fn set_note_naming(&mut self, note_naming: NoteNaming) {
        if self.note_naming != note_naming {
            self.note_naming = note_naming;
            self.invalidate_layout_cache();
        }
    }

    pub fn set_key_labels(&mut self, key_labels: bool) {
        if self.key_labels != key_labels {
            self.key_labels = key_labels;
            self.invalidate_layout_cache();
        }
    }

    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
    }

    fn invalidate_layout_cache(&mut self) {
        self.cache.clear();
        self.text_cache.clear();
        self.middle_c_dot = None;
    }

    /// Reupload instances to GPU
    #[profiling::function]
    fn rebuild_quad_cache(&mut self) {
        let instances = &mut self.cache;

        instances.push(QuadInstance {
            position: [self.pos.x, self.pos.y - 4.0],
            size: [self.layout.width, 4.0],
            color: [0.25, 0.02, 0.02, 1.0],
            ..Default::default()
        });

        // black_background
        instances.push(QuadInstance {
            position: self.pos.into(),
            size: [self.layout.width, self.layout.height],
            color: [0.0, 0.0, 0.0, 1.0],
            ..Default::default()
        });

        for key in self
            .layout
            .keys
            .iter()
            .filter(|key| key.kind().is_neutral())
        {
            let id = key.id();
            let color = self.key_states[id].color();

            instances.push(key_state::to_quad(key, color, self.pos));
        }

        for key in self.layout.keys.iter().filter(|key| key.kind().is_sharp()) {
            let id = key.id();
            let color = self.key_states[id].color();

            instances.push(key_state::to_quad(key, color, self.pos));
        }

        if let Some(dot) = self.middle_c_dot {
            let color = if !self.key_labels {
                [0.0, 0.0, 0.0, 150.0 / 255.0]
            } else if dark_text_on(self.key_states[dot.key_id].color()) {
                [0.0, 0.0, 0.0, 1.0]
            } else {
                [1.0, 1.0, 1.0, 1.0]
            };

            instances.push(QuadInstance {
                position: [dot.x, dot.y],
                size: [dot.size, dot.size],
                color,
                border_radius: [dot.size / 2.0; 4],
                ..Default::default()
            });
        }
    }

    #[profiling::function]
    fn rebuild_text_cache(&mut self) {
        if self.key_labels {
            self.rebuild_key_names();
        } else {
            self.rebuild_octave_labels();
        }
    }

    fn rebuild_octave_labels(&mut self) {
        let font_system = crate::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        let range_start = self.layout.range.start();
        for key in self.layout.keys.iter().filter(|key| key.note_id() == 0) {
            let x = self.pos.x + key.x();
            let y = self.pos.y;

            let w = key.width();
            let h = key.height();

            let size = if self.note_naming == NoteNaming::Letters {
                w * 0.7
            } else {
                // The name and the middle C dot above it stay below the black keys
                (w * 0.7).min((h - self.layout.sizing.sharp_height) / 2.0)
            };
            let top = y + h - size * 1.2;

            let note = range_start + key.id() as u8;
            let text = octave_label(self.note_naming, note);

            let mut buffer = glyphon::Buffer::new(font_system, glyphon::Metrics::new(size, size));
            buffer.set_size(Some(w), Some(h));
            buffer.set_wrap(glyphon::Wrap::None);
            buffer.set_text(
                &text,
                &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                label_shaping(&text),
                Some(glyphon::cosmic_text::Align::Center),
            );
            buffer.shape_until_scroll(font_system, false);

            if note == MIDDLE_C && self.note_naming != NoteNaming::Letters {
                let dot = size * 0.6;

                self.middle_c_dot = Some(MiddleCDot {
                    key_id: key.id(),
                    x: x + (w - 1.0 - dot) / 2.0,
                    y: top - dot,
                    size: dot,
                });
            }

            self.text_cache.push((
                key.id(),
                super::text::TextArea {
                    buffer,
                    left: x,
                    top,
                    scale: 1.0,
                    bounds: key_bounds(x, y, w, h),
                    default_color: glyphon::Color::rgba(0, 0, 0, 150),
                },
            ));
        }
    }

    fn name_sizes(&self, font_system: &mut glyphon::FontSystem) -> NameSizes {
        let naming = self.note_naming;
        let sizing = self.layout.sizing;

        let accidental_below = naming == NoteNaming::SolfegePt;
        let sharp_lines = if accidental_below { 2.0 } else { 1.0 };
        let sharp_em = TextRenderer::widest_label_em(
            font_system,
            SHARP_NOTES
                .map(|note| key_label(naming, note, accidental_below))
                .iter()
                .map(String::as_str),
        );

        let neutral_names = NEUTRAL_NOTES.map(|note| match naming {
            NoteNaming::Letters if note == 0 => octave_label(naming, MIDDLE_C),
            _ => naming.note_name(note).to_string(),
        });
        let neutral_em =
            TextRenderer::widest_label_em(font_system, neutral_names.iter().map(String::as_str));

        let strip = sizing.neutral_height - sizing.sharp_height;

        NameSizes {
            neutral: fit_label(sizing.neutral_width - 1.0, neutral_em).min(strip * 0.4),
            sharp: fit_label(sizing.sharp_width, sharp_em)
                .min(sizing.sharp_height * 0.9 / (sharp_lines + 0.35)),
            accidental_below,
        }
    }

    fn rebuild_key_names(&mut self) {
        let font_system = crate::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        let sizes = self.name_sizes(font_system);
        let range_start = self.layout.range.start();

        for key in self.layout.keys.iter() {
            let note = range_start + key.id() as u8;

            let x = self.pos.x + key.x();
            let y = self.pos.y;
            let w = key.width();
            let h = key.height();

            let text = if self.note_naming == NoteNaming::Letters && key.note_id() == 0 {
                octave_label(self.note_naming, note)
            } else {
                key_label(self.note_naming, note, sizes.accidental_below)
            };
            let lines = text.lines().count() as f32;
            let size = if key.kind().is_sharp() {
                sizes.sharp
            } else {
                sizes.neutral
            };
            let top = y + h - size * (lines + 0.35);

            let buffer = TextRenderer::shape_label(font_system, &text, size, Some(w));

            self.text_cache.push((
                key.id(),
                super::text::TextArea {
                    buffer,
                    left: x,
                    top,
                    scale: 1.0,
                    bounds: key_bounds(x, y, w, h),
                    default_color: text_color_on(self.key_states[key.id()].color()),
                },
            ));

            if note == MIDDLE_C && self.note_naming != NoteNaming::Letters {
                let dot = (size * 0.6).max(3.0);

                self.middle_c_dot = Some(MiddleCDot {
                    key_id: key.id(),
                    x: x + (w - 1.0 - dot) / 2.0,
                    y: top - dot - size * 0.15,
                    size: dot,
                });
            }
        }
    }

    #[profiling::function]
    pub fn update(&mut self, quads: &mut QuadRenderer, text: &mut TextRenderer) {
        // Labels first, the middle C dot needs their position
        if self.text_cache.is_empty() {
            self.rebuild_text_cache();
        }

        if self.cache.is_empty() {
            self.rebuild_quad_cache();
        }

        {
            profiling::scope!("push quads from cache");
            quads.layer().extend(&self.cache);
        }

        {
            profiling::scope!("push text from cache");
            let queue = text.queue_mut();
            for (key_id, area) in self.text_cache.iter() {
                let mut area = area.clone();

                if self.key_labels {
                    area.default_color = text_color_on(self.key_states[*key_id].color());
                }

                queue.push(area);
            }
        }
    }
}

fn octave_label(naming: NoteNaming, note: u8) -> String {
    match naming {
        NoteNaming::Letters => format!("C{}", (note / 12) as i8 - 1),
        // Octave numbers of solfège names differ between countries, so none is shown
        NoteNaming::SolfegePt => naming.note_name(note).to_string(),
    }
}

fn key_label(naming: NoteNaming, note: u8, accidental_below: bool) -> String {
    let name = naming.note_name(note);

    match name.strip_suffix('♭') {
        Some(name) if accidental_below => format!("{name}\n♭"),
        _ => name.to_string(),
    }
}

fn fit_label(key_width: f32, widest_em: f32) -> f32 {
    (key_width * 0.9 / widest_em).min(key_width * 0.7)
}

fn key_bounds(x: f32, y: f32, w: f32, h: f32) -> glyphon::TextBounds {
    glyphon::TextBounds {
        left: x.round() as i32,
        top: y.round() as i32,
        right: x.round() as i32 + w.round() as i32,
        bottom: y.round() as i32 + h.round() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keyboard(width: f32, height: f32, range: KeyboardRange) -> KeyboardRenderer {
        let sizing = piano_layout::Sizing::new(width / range.white_count() as f32, height * 0.2);
        KeyboardRenderer::new(piano_layout::KeyboardLayout::from_range(sizing, range))
    }

    #[test]
    fn note_events_keep_the_labels() {
        let mut keyboard = keyboard(1080.0, 720.0, KeyboardRange::standard_88_keys());
        keyboard.rebuild_text_cache();
        keyboard.invalidate_cache();

        assert_eq!(keyboard.text_cache.len(), 8);
    }

    #[test]
    fn middle_c_dot_stays_below_the_black_keys() {
        let mut keyboard = keyboard(3440.0, 620.0, KeyboardRange::standard_88_keys());
        keyboard.set_note_naming(NoteNaming::SolfegePt);
        keyboard.rebuild_text_cache();

        let dot = keyboard.middle_c_dot.unwrap();
        assert!(dot.y >= keyboard.layout.sizing.sharp_height);
    }

    #[test]
    fn solfege_labels_fit_their_keys() {
        for (width, height) in [(1080.0, 720.0), (3440.0, 620.0)] {
            let mut keyboard = keyboard(width, height, KeyboardRange::new(36..=96));
            keyboard.set_key_labels(true);
            keyboard.set_note_naming(NoteNaming::SolfegePt);
            keyboard.rebuild_text_cache();

            assert_eq!(keyboard.text_cache.len(), 61);
            for (id, area) in keyboard.text_cache.iter() {
                let key = &keyboard.layout.keys[*id];
                let (text_width, _) = TextRenderer::measure(&area.buffer);
                assert!(text_width <= key.width(), "key {id} is too narrow");
                assert!(
                    area.top >= keyboard.pos.y,
                    "key {id} label starts above the key"
                );
            }
        }
    }
}
