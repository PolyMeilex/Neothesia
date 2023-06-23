use crate::{utils::Point, TransformUniform, Uniform};

use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use piano_math::range::KeyboardRange;
use wgpu_glyph::{GlyphBrush, Section};

mod key_state;
pub use key_state::KeyState;
use wgpu_jumpstart::Gpu;

pub struct KeyboardRenderer {
    pos: Point<f32>,

    key_states: Vec<KeyState>,

    quad_pipeline: QuadPipeline,
    should_reupload: bool,

    layout: piano_math::KeyboardLayout,
}

impl KeyboardRenderer {
    pub fn new(
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        layout: piano_math::KeyboardLayout,
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

    pub fn layout(&mut self) -> &piano_math::KeyboardLayout {
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

    pub fn update(&mut self, queue: &wgpu::Queue, brush: &mut GlyphBrush<()>) {
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

            brush.queue(Section {
                screen_position: (x + w / 2.0, y + h - size * 1.2),
                text: vec![wgpu_glyph::Text::new(&format!("C{}", id + 1))
                    .with_color([0.6, 0.6, 0.6, 1.0])
                    .with_scale(size)],
                bounds: (w, f32::INFINITY),
                layout: wgpu_glyph::Layout::default()
                    .h_align(wgpu_glyph::HorizontalAlign::Center)
                    .v_align(wgpu_glyph::VerticalAlign::Top),
            })
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
