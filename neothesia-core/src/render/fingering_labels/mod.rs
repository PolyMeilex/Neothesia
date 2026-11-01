use midi_file::MidiNote;

use crate::{fingering::Fingering, utils::Point};

use super::{KeyboardRenderer, TextRenderer, waterfall::NoteList};

const AUTO_COLOR: glyphon::Color = glyphon::Color::rgb(255, 255, 255);
const OVERRIDE_COLOR: glyphon::Color = glyphon::Color::rgb(255, 205, 60);
const NEUTRAL_KEY_COLOR: glyphon::Color = glyphon::Color::rgb(40, 40, 40);

#[derive(Default)]
struct LabelsCache {
    /// Labels for fingers 1 to 5
    labels: Option<[(glyphon::Buffer, f32); 5]>,
    size: f32,
}

impl LabelsCache {
    #[profiling::function]
    fn get(&mut self, size: f32) -> &[(glyphon::Buffer, f32); 5] {
        let font_system = crate::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        if self.labels.is_none() || self.size != size {
            let labels = std::array::from_fn(|id| {
                let mut buffer =
                    glyphon::Buffer::new(font_system, glyphon::Metrics::new(size, size));
                buffer.set_wrap(glyphon::Wrap::None);
                buffer.set_text(
                    &(id + 1).to_string(),
                    &glyphon::Attrs::new()
                        .family(glyphon::Family::SansSerif)
                        .weight(glyphon::Weight::BOLD),
                    glyphon::Shaping::Basic,
                    None,
                );
                buffer.shape_until_scroll(font_system, false);

                let (text_width, _) = TextRenderer::measure(&buffer);
                (buffer, text_width)
            });

            self.labels = Some(labels);
            self.size = size;
        }

        self.labels.as_ref().unwrap()
    }
}

/// Finger numbers on falling notes and on currently played keys
pub struct FingeringLabels {
    pos: Point<f32>,
    notes: NoteList,
    fingering: Fingering,
    on_keys: bool,
    /// Leave room for note name labels at the bottom of each note
    above_note_labels: bool,
    labels_cache: LabelsCache,
    /// Rendered above the waterfall
    notes_text: TextRenderer,
    /// Rendered above the keyboard
    keys_text: TextRenderer,
}

impl FingeringLabels {
    pub fn new(
        pos: Point<f32>,
        notes: &NoteList,
        fingering: Fingering,
        on_keys: bool,
        above_note_labels: bool,
        notes_text: TextRenderer,
        keys_text: TextRenderer,
    ) -> Self {
        Self {
            pos,
            notes: notes.clone(),
            fingering,
            on_keys,
            above_note_labels,
            labels_cache: LabelsCache::default(),
            notes_text,
            keys_text,
        }
    }

    pub fn set_pos(&mut self, pos: Point<f32>) {
        self.pos = pos;
    }

    pub fn set_fingering(&mut self, fingering: Fingering) {
        self.fingering = fingering;
    }

    pub fn fingering(&self) -> &Fingering {
        &self.fingering
    }

    #[profiling::function]
    pub fn update(
        &mut self,
        physical_size: dpi::PhysicalSize<u32>,
        scale: f32,
        keyboard: &KeyboardRenderer,
        animation_speed: f32,
        time: f32,
    ) {
        let layout = keyboard.layout();
        let range_start = layout.range.start() as usize;
        let label_size = layout.sizing.sharp_width;
        let keyboard_y = keyboard.pos().y;
        let pos_y = self.pos.y;
        let show_on_keys = self.on_keys;

        let labels = self.labels_cache.get(label_size);
        let animation_speed = animation_speed / scale;
        let fingering = &self.fingering;
        let y_offset = if self.above_note_labels {
            label_size * 2.0
        } else {
            label_size
        };

        let visible = |note: &&MidiNote| layout.range.contains(note.note) && note.channel != 9;

        let on_notes = self
            .notes
            .inner
            .iter()
            .filter(visible)
            .map(|note| {
                let y = pos_y - (note.start.as_secs_f32() - time) * animation_speed - y_offset;
                (note, y)
            })
            // Stop iteration once we reach top of the screen
            .take_while(|(_note, y)| *y > 0.0)
            .skip_while(|(_note, y)| *y > keyboard_y)
            .filter_map(|(note, y)| {
                let f = fingering.get(note)?;
                let (buffer, text_width) = &labels[f.finger as usize - 1];

                let key = &layout.keys[note.note as usize - range_start];
                let x = key.x() + (key.width() - text_width) / 2.0;
                let color = if f.overridden {
                    OVERRIDE_COLOR
                } else {
                    AUTO_COLOR
                };

                Some(text_area(buffer, x, y, color))
            });

        let on_keys = self
            .notes
            .inner
            .iter()
            .filter(|_| show_on_keys)
            .take_while(|note| note.start.as_secs_f32() <= time)
            .filter(|note| note.end.as_secs_f32() > time)
            .filter(visible)
            .filter_map(|note| {
                let f = fingering.get(note)?;
                let (buffer, text_width) = &labels[f.finger as usize - 1];

                let key = &layout.keys[note.note as usize - range_start];
                let x = key.x() + (key.width() - text_width) / 2.0;
                let (y, color) = if key.kind().is_sharp() {
                    (keyboard_y + key.height() - label_size * 1.4, AUTO_COLOR)
                } else {
                    // Above the note name label of the key
                    (
                        keyboard_y + key.height() - key.width() * 0.7 * 1.2 - label_size * 1.3,
                        NEUTRAL_KEY_COLOR,
                    )
                };

                Some(text_area(buffer, x, y, color))
            });

        self.notes_text
            .update_from_iter(physical_size, scale, on_notes);
        self.keys_text
            .update_from_iter(physical_size, scale, on_keys);
    }

    /// Call right after the waterfall
    pub fn render_notes<'rpass>(
        &'rpass self,
        render_pass: &mut wgpu_jumpstart::RenderPass<'rpass>,
    ) {
        self.notes_text.render(render_pass);
    }

    /// Call after the keyboard
    pub fn render_keys<'rpass>(&'rpass self, render_pass: &mut wgpu_jumpstart::RenderPass<'rpass>) {
        self.keys_text.render(render_pass);
    }
}

fn text_area(
    buffer: &glyphon::Buffer,
    left: f32,
    top: f32,
    color: glyphon::Color,
) -> glyphon::TextArea<'_> {
    glyphon::TextArea {
        buffer,
        left,
        top,
        scale: 1.0,
        bounds: glyphon::TextBounds {
            left: 0,
            top: 0,
            right: i32::MAX,
            bottom: i32::MAX,
        },
        default_color: color,
        custom_glyphs: &[],
    }
}
