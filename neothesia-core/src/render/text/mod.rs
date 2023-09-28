use std::sync::Arc;

use wgpu_jumpstart::Gpu;

pub use glyphon;

pub struct TextArea {
    pub buffer: glyphon::Buffer,
    /// The left edge of the buffer.
    pub left: f32,
    /// The top edge of the buffer.
    pub top: f32,
    /// The scaling to apply to the buffer.
    pub scale: f32,
    /// The visible bounds of the text area. This is used to clip the text and doesn't have to
    /// match the `left` and `top` values.
    pub bounds: glyphon::TextBounds,
    // The default color of the text area.
    pub default_color: glyphon::Color,
}

pub struct TextRenderer {
    font_system: glyphon::FontSystem,
    cache: glyphon::SwashCache,
    atlas: glyphon::TextAtlas,
    text_renderer: glyphon::TextRenderer,

    queue: Vec<TextArea>,
}

impl TextRenderer {
    pub fn new(gpu: &Gpu) -> Self {
        let font_system = glyphon::FontSystem::new_with_fonts(
            [glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                "./Roboto-Regular.ttf"
            )))]
            .into_iter(),
        );

        let cache = glyphon::SwashCache::new();
        let mut atlas = glyphon::TextAtlas::new(&gpu.device, &gpu.queue, gpu.texture_format);
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            font_system,
            cache,
            atlas,
            text_renderer,
            queue: Vec::new(),
        }
    }

    pub fn font_system(&mut self) -> &mut glyphon::FontSystem {
        &mut self.font_system
    }

    pub fn atlas(&mut self) -> &mut glyphon::TextAtlas {
        &mut self.atlas
    }

    pub fn queue(&mut self, area: TextArea) {
        self.queue.push(area);
    }

    pub fn queue_text(&mut self, text: &str) {
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(15.0, 15.0));
        buffer.set_size(&mut self.font_system, f32::MAX, f32::MAX);
        buffer.set_text(
            &mut self.font_system,
            text,
            glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system);

        #[cfg(debug_assertions)]
        let top = 20.0;
        #[cfg(not(debug_assertions))]
        let top = 5.0;

        self.queue(TextArea {
            buffer,
            left: 0.0,
            top,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    pub fn queue_fps(&mut self, fps: f64) {
        let text = format!("FPS: {}", fps.round() as u32);
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(15.0, 15.0));
        buffer.set_size(&mut self.font_system, f32::MAX, f32::MAX);
        buffer.set_text(
            &mut self.font_system,
            &text,
            glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system);

        self.queue(TextArea {
            buffer,
            left: 0.0,
            top: 5.0,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    pub fn update(&mut self, logical_size: (u32, u32), gpu: &Gpu) {
        let elements = self.queue.iter().map(|area| glyphon::TextArea {
            buffer: &area.buffer,
            left: area.left,
            top: area.top,
            scale: area.scale,
            bounds: area.bounds,
            default_color: area.default_color,
        });

        self.text_renderer
            .prepare(
                &gpu.device,
                &gpu.queue,
                &mut self.font_system,
                &mut self.atlas,
                glyphon::Resolution {
                    width: logical_size.0,
                    height: logical_size.1,
                },
                elements,
                &mut self.cache,
            )
            .unwrap();

        self.queue.clear();
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.text_renderer.render(&self.atlas, render_pass).unwrap();
    }
}
