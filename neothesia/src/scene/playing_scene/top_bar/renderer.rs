use neothesia_core::render::{QuadPipeline, TextRenderer};

use crate::scene::playing_scene::LAYER_FG;

pub struct NuonRenderer<'a> {
    pub quads: &'a mut QuadPipeline,
    pub text: &'a mut TextRenderer,
}

impl nuon::Renderer for NuonRenderer<'_> {
    #[profiling::function]
    fn rounded_quad(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: nuon::Color,
        border_radius: [f32; 4],
    ) {
        self.quads.push(
            LAYER_FG,
            neothesia_core::render::QuadInstance {
                position: [x, y],
                size: [w, h],
                color: wgpu_jumpstart::Color::new(color.r, color.g, color.b, color.a)
                    .into_linear_rgba(),
                border_radius,
            },
        )
    }

    #[profiling::function]
    fn icon(&mut self, x: f32, y: f32, size: f32, icon: &str) {
        self.text.queue_icon(x, y, size, icon)
    }

    #[profiling::function]
    fn centered_text_bold(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, text: &str) {
        let buffer = self.text.gen_buffer_bold(size, text);
        self.text.queue_buffer_centered(x, y, w, h, buffer);
    }

    #[profiling::function]
    fn centered_text(&mut self, x: f32, y: f32, w: f32, h: f32, size: f32, text: &str) {
        let buffer = self.text.gen_buffer(size, text);
        self.text.queue_buffer_centered(x, y, w, h, buffer);
    }
}
