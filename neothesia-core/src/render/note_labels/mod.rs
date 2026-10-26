use crate::{
    config::{Config, NoteNaming},
    utils::Point,
};

use super::{KeyboardRenderer, TextRenderer, waterfall::NoteList};

pub(super) const SHARP_NOTES: [u8; 5] = [1, 3, 6, 8, 10];

struct Label {
    buffer: glyphon::Buffer,
    offset_x: f32,
    size: f32,
}

#[derive(Default)]
struct LabelsCache {
    labels: Option<[Label; 12]>,
    neutral_width: f32,
}

impl LabelsCache {
    #[profiling::function]
    fn get(&mut self, keyboard: &KeyboardRenderer, naming: NoteNaming) -> &[Label; 12] {
        let font_system = crate::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        let sharp_width = keyboard.layout().sizing.sharp_width;
        let neutral_width = keyboard.layout().sizing.neutral_width;

        if self.labels.is_none() || self.neutral_width != neutral_width {
            let sharp_size = if naming == NoteNaming::Letters {
                sharp_width
            } else {
                let widest = TextRenderer::widest_label_em(
                    font_system,
                    SHARP_NOTES.map(|note| naming.note_name(note)),
                );
                sharp_width * (1.35 / widest).min(1.0)
            };

            let labels = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(|note| {
                let (size, note_width) = if SHARP_NOTES.contains(&note) {
                    (sharp_size, sharp_width)
                } else {
                    (sharp_width, neutral_width)
                };

                let buffer = TextRenderer::shape_label(font_system, naming.note_name(note), size);
                let (text_width, _) = TextRenderer::measure(&buffer);

                Label {
                    buffer,
                    offset_x: (note_width - text_width) / 2.0,
                    size,
                }
            });

            self.labels = Some(labels);
            self.neutral_width = neutral_width;
        }

        self.labels.as_ref().unwrap()
    }
}

pub struct NoteLabels {
    pos: Point<f32>,
    notes: NoteList,
    naming: NoteNaming,
    labels_cache: LabelsCache,
    text_renderer: TextRenderer,
}

impl NoteLabels {
    pub fn new(
        pos: Point<f32>,
        notes: &NoteList,
        config: &Config,
        text_renderer: TextRenderer,
    ) -> Self {
        Self {
            pos,
            notes: notes.clone(),
            naming: config.note_naming(),
            labels_cache: LabelsCache::default(),
            text_renderer,
        }
    }

    pub fn set_pos(&mut self, pos: Point<f32>) {
        self.pos = pos;
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

        let labels = self.labels_cache.get(keyboard, self.naming);
        let animation_speed = animation_speed / scale;

        let iter = self
            .notes
            .inner
            .iter()
            .filter(|note| layout.range.contains(note.note) && note.channel != 9)
            .map(|note| {
                let label = &labels[(note.note % 12) as usize];

                let x = layout.keys[note.note as usize - range_start].x() + label.offset_x;
                let y =
                    self.pos.y - (note.start.as_secs_f32() - time) * animation_speed - label.size;

                (&label.buffer, x, y)
            })
            // Stop iteration once we reach top of the screen
            .take_while(|(_buffer, _x, y)| *y > 0.0)
            // TODO: Cache last note idx to skip this NoOp skip iteration
            .skip_while(|(_buffer, _x, y)| *y > keyboard.pos().y)
            .map(|(buffer, left, top)| glyphon::TextArea {
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
                default_color: glyphon::Color::rgb(255, 255, 255),
                custom_glyphs: &[],
            });

        self.text_renderer
            .update_from_iter(physical_size, scale, iter);
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu_jumpstart::RenderPass<'rpass>) {
        self.text_renderer.render(render_pass);
    }
}
