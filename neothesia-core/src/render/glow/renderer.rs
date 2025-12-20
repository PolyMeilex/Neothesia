use std::time::Duration;

use wgpu_jumpstart::{Color, Gpu, TransformUniform, Uniform};

use super::{GlowInstance, GlowPipeline};

struct GlowState {
    time: f32,
}

impl GlowState {
    fn size(&self) -> f32 {
        150.0 + self.time.sin() * 10.0
    }

    fn update(&mut self, delta: Duration) {
        self.time += delta.as_secs_f32() * 5.0;
    }

    fn calc_color(&self, color: Color) -> [f32; 4] {
        let mut color = color.into_linear_rgba();
        let v = 0.2 * self.time.cos().abs();
        let v = v.min(1.0);
        color[0] += v;
        color[1] += v;
        color[2] += v;
        color[3] = 0.2;
        color
    }
}

pub struct GlowRenderer {
    pipeline: GlowPipeline,
    states: Vec<GlowState>,
}

impl GlowRenderer {
    pub fn new(
        gpu: &Gpu,
        transform: &Uniform<TransformUniform>,
        layout: &piano_layout::KeyboardLayout,
    ) -> Self {
        let pipeline = GlowPipeline::new(gpu, transform);

        let states: Vec<GlowState> = layout
            .range
            .iter()
            .map(|_| GlowState { time: 0.0 })
            .collect();

        Self { pipeline, states }
    }

    pub fn prepare(&mut self) {
        self.pipeline.prepare();
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        self.pipeline.render(render_pass);
    }

    pub fn clear(&mut self) {
        self.pipeline.clear();
    }

    pub fn push(
        &mut self,
        id: usize,
        color: Color,
        key_x: f32,
        key_y: f32,
        key_w: f32,
        delta: Duration,
    ) {
        let state = &mut self.states[id];
        state.update(delta);

        let color = state.calc_color(color);
        let glow_w = state.size();
        let glow_h = glow_w;

        self.pipeline.instances().push(GlowInstance {
            position: [key_x - glow_w / 2.0 + key_w / 2.0, key_y - glow_w / 2.0],
            size: [glow_w, glow_h],
            color,
        });
    }
}
