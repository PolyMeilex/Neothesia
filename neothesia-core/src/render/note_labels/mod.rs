use crate::{config::NoteNaming, utils::Point};

use super::{KeyboardRenderer, TextRenderer, waterfall::NoteList};

#[derive(Default)]
struct LabelsCache {
    labels: Option<[(glyphon::Buffer, f32); 12]>,
    neutral_width: f32,
    naming: NoteNaming,
}

impl LabelsCache {
    #[profiling::function]
    fn get(&mut self, keyboard: &KeyboardRenderer) -> &[(glyphon::Buffer, f32); 12] {
        let font_system = crate::font_system::font_system();
        let font_system = &mut font_system.borrow_mut();

        let sharp_width = keyboard.layout().sizing.sharp_width;
        let neutral_width = keyboard.layout().sizing.neutral_width;

        if self.labels.is_none() || self.neutral_width != neutral_width {
            let label_width = sharp_width;

            let labels = std::array::from_fn(|note_id| {
                let mut buffer = glyphon::Buffer::new(
                    font_system,
                    glyphon::Metrics::new(label_width, label_width),
                );
                buffer.set_wrap(glyphon::Wrap::None);
                buffer.set_text(
                    self.naming.note_name(note_id as u8),
                    &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                    glyphon::Shaping::Basic,
                    None,
                );
                buffer.shape_until_scroll(font_system, false);

                let (text_width, _) = TextRenderer::measure(&buffer);
                (buffer, text_width)
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
    labels_cache: LabelsCache,
    text_renderer: TextRenderer,
}

impl NoteLabels {
    pub fn new(
        pos: Point<f32>,
        notes: &NoteList,
        naming: NoteNaming,
        text_renderer: TextRenderer,
    ) -> Self {
        Self {
            pos,
            notes: notes.clone(),
            labels_cache: LabelsCache {
                naming,
                ..Default::default()
            },
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
        let label_width = layout.sizing.sharp_width;

        let labels = self.labels_cache.get(keyboard);
        let animation_speed = animation_speed / scale;

        let iter = self
            .notes
            .inner
            .iter()
            .filter(|note| layout.range.contains(note.note) && note.channel != 9)
            .map(|note| {
                let (buffer, text_width) = &labels[(note.note % 12) as usize];

                let key = &layout.keys[note.note as usize - range_start];
                let x = key.x() + (key.width() - text_width) / 2.0;
                let y =
                    self.pos.y - (note.start.as_secs_f32() - time) * animation_speed - label_width;

                (buffer, x, y)
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
