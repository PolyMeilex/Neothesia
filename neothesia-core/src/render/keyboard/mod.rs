use crate::{
    config::NoteNaming,
    render::{QuadInstance, QuadRenderer},
    utils::Point,
};

use piano_layout::range::KeyboardRange;

mod key_state;
pub use key_state::KeyState;

use super::{TextRenderer, text::label_shaping};

const MIDDLE_C: u8 = 60;

#[derive(Clone, Copy)]
struct MiddleCDot {
    x: f32,
    y: f32,
    size: f32,
}

pub struct KeyboardRenderer {
    pos: Point<f32>,

    key_states: Vec<KeyState>,

    layout: piano_layout::KeyboardLayout,
    note_naming: NoteNaming,

    cache: Vec<QuadInstance>,
    text_cache: Vec<super::text::TextArea>,
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
            instances.push(QuadInstance {
                position: [dot.x, dot.y],
                size: [dot.size, dot.size],
                color: [0.0, 0.0, 0.0, 150.0 / 255.0],
                border_radius: [dot.size / 2.0; 4],
                ..Default::default()
            });
        }
    }

    #[profiling::function]
    fn rebuild_text_cache(&mut self) {
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
                    x: x + (w - 1.0 - dot) / 2.0,
                    y: top - dot,
                    size: dot,
                });
            }

            self.text_cache.push(super::text::TextArea {
                buffer,
                left: x,
                top,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: x.round() as i32,
                    top: y.round() as i32,
                    right: x.round() as i32 + w.round() as i32,
                    bottom: y.round() as i32 + h.round() as i32,
                },
                default_color: glyphon::Color::rgba(0, 0, 0, 150),
            });
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
            text.queue_mut().extend_from_slice(&self.text_cache);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_events_keep_the_labels() {
        let range = KeyboardRange::standard_88_keys();
        let sizing = piano_layout::Sizing::new(1080.0 / range.white_count() as f32, 144.0);
        let mut keyboard =
            KeyboardRenderer::new(piano_layout::KeyboardLayout::from_range(sizing, range));
        keyboard.rebuild_text_cache();
        keyboard.invalidate_cache();

        assert_eq!(keyboard.text_cache.len(), 8);
    }

    #[test]
    fn middle_c_dot_stays_below_the_black_keys() {
        let range = KeyboardRange::standard_88_keys();
        let sizing = piano_layout::Sizing::new(3440.0 / range.white_count() as f32, 124.0);
        let mut keyboard =
            KeyboardRenderer::new(piano_layout::KeyboardLayout::from_range(sizing, range));
        keyboard.set_note_naming(NoteNaming::SolfegePt);
        keyboard.rebuild_text_cache();

        let dot = keyboard.middle_c_dot.unwrap();
        assert!(dot.y >= keyboard.layout.sizing.sharp_height);
    }
}
