use crate::{
    render::{QuadInstance, QuadRenderer},
    utils::Point,
};

use piano_layout::range::KeyboardRange;

mod key_state;
pub use key_state::KeyState;

use super::TextRenderer;

pub struct KeyboardRenderer {
    pos: Point<f32>,

    key_states: Vec<KeyState>,

    layout: piano_layout::KeyboardLayout,

    cache: Vec<QuadInstance>,
    text_cache: Vec<super::text::TextArea>,
}

impl KeyboardRenderer {
    pub fn new(layout: piano_layout::KeyboardLayout) -> Self {
        let key_states: Vec<KeyState> = layout
            .range
            .iter()
            .map(|id| KeyState::new(id.is_black()))
            .collect();

        let cache = Vec::with_capacity(key_states.len() + 1);

        Self {
            pos: Default::default(),

            key_states,

            layout,
            cache,
            text_cache: Vec::new(),
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
        self.invalidate_cache();
    }

    pub fn layout(&self) -> &piano_layout::KeyboardLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: piano_layout::KeyboardLayout) {
        self.layout = layout;
        self.invalidate_cache();
    }

    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
        self.text_cache.clear();
    }

    /// Reupload instances to GPU
    #[profiling::function]
    fn rebuild_quad_cache(&mut self) {
        let instances = &mut self.cache;

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
    }

    #[profiling::function]
    fn rebuild_text_cache(&mut self) {
        let font_system = crate::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        let range_start = self.layout.range.start() as usize;
        for key in self.layout.keys.iter().filter(|key| key.note_id() == 0) {
            let x = self.pos.x + key.x();
            let y = self.pos.y;

            let w = key.width();
            let h = key.height();

            let size = w * 0.7;

            let oct_number = (key.id() + range_start) / 12;

            let mut buffer = glyphon::Buffer::new(font_system, glyphon::Metrics::new(size, size));
            buffer.set_size(font_system, Some(w), Some(h));
            buffer.set_wrap(font_system, glyphon::Wrap::None);
            buffer.set_text(
                font_system,
                &format!("C{}", oct_number as i8 - 1),
                &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                glyphon::Shaping::Basic,
                Some(glyphon::cosmic_text::Align::Center),
            );
            buffer.shape_until_scroll(font_system, false);

            self.text_cache.push(super::text::TextArea {
                buffer,
                left: x,
                top: y + h - size * 1.2,
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
        if self.cache.is_empty() {
            self.rebuild_quad_cache();
        }

        if self.text_cache.is_empty() {
            self.rebuild_text_cache();
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
