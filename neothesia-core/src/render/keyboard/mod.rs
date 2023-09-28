use crate::{
    render::{QuadInstance, QuadPipeline},
    utils::Point,
    TransformUniform, Uniform,
};

use piano_math::range::KeyboardRange;

mod key_state;
pub use key_state::KeyState;
use wgpu_jumpstart::Gpu;

use super::TextRenderer;

pub struct KeyboardRenderer {
    pos: Point<f32>,

    key_states: Vec<KeyState>,

    quad_pipeline: QuadPipeline,
    should_reupload: bool,

    layout: piano_math::KeyboardLayout,
    vertical_guidelines: bool,
}

impl KeyboardRenderer {
    pub fn new(
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        layout: piano_math::KeyboardLayout,
        vertical_guidelines: bool,
    ) -> Self {
        let quad_pipeline = QuadPipeline::new(gpu, transform_uniform);
        let key_states: Vec<KeyState> = layout
            .range
            .iter()
            .map(|id| KeyState::new(id.is_black()))
            .collect();

        Self {
            pos: Default::default(),

            key_states,

            quad_pipeline,
            should_reupload: false,

            layout,
            vertical_guidelines,
        }
    }

    pub fn reset_notes(&mut self) {
        for key in self.key_states.iter_mut() {
            key.pressed_by_file_off();
        }
        self.queue_reupload();
    }

    pub fn range(&self) -> &KeyboardRange {
        &self.layout.range
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
        self.queue_reupload();
    }

    pub fn layout(&self) -> &piano_math::KeyboardLayout {
        &self.layout
    }

    pub fn set_layout(&mut self, layout: piano_math::KeyboardLayout) {
        self.layout = layout;
        self.queue_reupload();
    }

    pub fn queue_reupload(&mut self) {
        self.should_reupload = true;
    }

    /// Reupload instances to GPU
    fn reupload(&mut self, queue: &wgpu::Queue) {
        self.quad_pipeline.with_instances_mut(queue, |instances| {
            instances.clear();

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

                if self.vertical_guidelines {
                    // Horizontal guides
                    // TODO: Does not really fit in keyboard renderer
                    if key.note_id() == 0 || key.note_id() == 5 {
                        let x = self.pos.x + key.x();
                        let y = 0.0;

                        let w = 1.0;
                        let h = f32::MAX;

                        let color = if key.note_id() == 0 {
                            [0.2, 0.2, 0.2, 1.0]
                        } else {
                            [0.05, 0.05, 0.05, 1.0]
                        };

                        instances.push(QuadInstance {
                            position: [x, y],
                            size: [w, h],
                            color,
                            border_radius: [0.0, 0.0, 0.0, 0.0],
                        });
                    }
                }

                instances.push(key_state::to_quad(key, color, self.pos));
            }

            for key in self.layout.keys.iter().filter(|key| key.kind().is_sharp()) {
                let id = key.id();
                let color = self.key_states[id].color();

                instances.push(key_state::to_quad(key, color, self.pos));
            }
        });
        self.should_reupload = false;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, text: &mut TextRenderer) {
        if self.should_reupload {
            self.reupload(queue);
        }

        for (id, key) in self
            .layout
            .keys
            .iter()
            .filter(|key| key.note_id() == 0)
            .enumerate()
        {
            let x = self.pos.x + key.x();
            let y = self.pos.y;

            let w = key.width();
            let h = key.height();

            let size = w * 0.7;

            let mut buffer =
                glyphon::Buffer::new(text.font_system(), glyphon::Metrics::new(size, size));
            buffer.set_size(text.font_system(), w, h);
            buffer.set_wrap(text.font_system(), glyphon::Wrap::None);
            buffer.set_text(
                text.font_system(),
                &format!("C{}", id + 1),
                glyphon::Attrs::new().family(glyphon::Family::SansSerif),
                glyphon::Shaping::Basic,
            );
            buffer.lines[0].set_align(Some(glyphon::cosmic_text::Align::Center));
            buffer.shape_until_scroll(text.font_system());

            text.queue(super::text::TextArea {
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
                default_color: glyphon::Color::rgb(153, 153, 153),
            });
        }
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.quad_pipeline.render(transform_uniform, render_pass);
    }
}
