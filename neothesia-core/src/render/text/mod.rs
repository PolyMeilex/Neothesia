use std::{cell::RefCell, rc::Rc, sync::Arc};

use wgpu_jumpstart::Gpu;

pub use glyphon;

#[derive(Debug, Clone)]
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

struct TextShared {
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    swash_cache: glyphon::SwashCache,
}

pub struct TextRendererInstance {
    text_renderer: glyphon::TextRenderer,
    queue: Vec<TextArea>,
    shared: Rc<RefCell<TextShared>>,
}

impl TextRendererInstance {
    pub fn queue(&mut self, area: TextArea) {
        self.queue.push(area);
    }

    pub fn update(
        &mut self,
        logical_size: (u32, u32),
        text_renderer: &mut TextRenderer,
        gpu: &mut Gpu,
    ) {
        let shared = &mut *self.shared.borrow_mut();
        let elements = self.queue.iter().map(|area| glyphon::TextArea {
            buffer: &area.buffer,
            left: area.left,
            top: area.top,
            scale: area.scale,
            bounds: area.bounds,
            default_color: area.default_color,
        });

        shared.viewport.update(
            &gpu.queue,
            glyphon::Resolution {
                width: logical_size.0,
                height: logical_size.1,
            },
        );

        self.text_renderer
            .prepare(
                &gpu.device,
                &gpu.queue,
                &mut gpu.encoder,
                &mut text_renderer.font_system,
                &mut shared.atlas,
                &shared.viewport,
                elements,
                &mut shared.swash_cache,
            )
            .unwrap();

        self.queue.clear();
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        let shared = self.shared.borrow();
        self.text_renderer
            .render(&shared.atlas, &shared.viewport, render_pass)
            .unwrap();
    }
}

pub struct TextRenderer {
    shared: Rc<RefCell<TextShared>>,

    font_system: glyphon::FontSystem,
    _cache: glyphon::Cache,
    text_renderer: glyphon::TextRenderer,

    queue: Vec<TextArea>,
}

impl TextRenderer {
    pub fn new(gpu: &Gpu) -> Self {
        let font_system = glyphon::FontSystem::new_with_fonts([
            glyphon::fontdb::Source::Binary(Arc::new(include_bytes!("./Roboto-Regular.ttf"))),
            glyphon::fontdb::Source::Binary(Arc::new(include_bytes!(
                "../../../../neothesia/src/iced_utils/bootstrap-icons.ttf"
            ))),
        ]);

        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(&gpu.device);
        let mut atlas =
            glyphon::TextAtlas::new(&gpu.device, &gpu.queue, &cache, gpu.texture_format);
        let text_renderer = glyphon::TextRenderer::new(
            &mut atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );

        let viewport = glyphon::Viewport::new(&gpu.device, &cache);

        Self {
            shared: Rc::new(RefCell::new(TextShared {
                viewport,
                atlas,
                swash_cache,
            })),
            font_system,
            _cache: cache,
            text_renderer,
            queue: Vec::new(),
        }
    }

    pub fn new_renderer(&mut self, gpu: &Gpu) -> TextRendererInstance {
        let text_renderer = glyphon::TextRenderer::new(
            &mut self.shared.borrow_mut().atlas,
            &gpu.device,
            wgpu::MultisampleState::default(),
            None,
        );

        TextRendererInstance {
            text_renderer,
            queue: Vec::new(),
            shared: self.shared.clone(),
        }
    }

    pub fn font_system(&mut self) -> &mut glyphon::FontSystem {
        &mut self.font_system
    }

    pub fn queue(&mut self, area: TextArea) {
        self.queue.push(area);
    }

    pub fn queue_text(&mut self, text: &str) {
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(15.0, 15.0));
        buffer.set_size(&mut self.font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(
            &mut self.font_system,
            text,
            glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

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

    pub fn measure(buffer: &glyphon::cosmic_text::Buffer) -> (f32, f32) {
        buffer
            .layout_runs()
            .fold((0.0, 0.0), |(width, height), run| {
                (run.line_w.max(width), height + run.line_height)
            })
    }

    pub fn gen_buffer(&mut self, size: f32, text: &str) -> glyphon::Buffer {
        self.gen_buffer_with_attr(
            size,
            text,
            glyphon::Attrs::new().family(glyphon::Family::Name("Roboto")),
        )
    }

    pub fn gen_buffer_bold(&mut self, size: f32, text: &str) -> glyphon::Buffer {
        self.gen_buffer_with_attr(
            size,
            text,
            glyphon::Attrs::new()
                .family(glyphon::Family::Name("Roboto"))
                .weight(glyphon::Weight::BOLD),
        )
    }

    pub fn gen_buffer_with_attr(
        &mut self,
        size: f32,
        text: &str,
        attrs: glyphon::Attrs,
    ) -> glyphon::Buffer {
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(size, size));
        buffer.set_size(&mut self.font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(&mut self.font_system, text, attrs, glyphon::Shaping::Basic);
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    pub fn queue_buffer(&mut self, x: f32, y: f32, buffer: glyphon::Buffer) {
        self.queue(TextArea {
            buffer,
            left: x,
            top: y,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    pub fn queue_buffer_centered(
        &mut self,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        buffer: glyphon::Buffer,
    ) {
        let (text_w, text_h) = Self::measure(&buffer);

        self.queue(TextArea {
            buffer,
            left: x + w / 2.0 - text_w / 2.0,
            top: y + h / 2.0 - text_h / 2.0,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    pub fn queue_icon(&mut self, x: f32, y: f32, size: f32, icon: &str) {
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(size, size));
        buffer.set_size(&mut self.font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(
            &mut self.font_system,
            icon,
            glyphon::Attrs::new().family(glyphon::Family::Name("bootstrap-icons")),
            glyphon::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.queue(TextArea {
            buffer,
            left: x,
            top: y,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    pub fn queue_fps(&mut self, fps: f64) {
        let text = format!("FPS: {}", fps.round() as u32);
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(15.0, 15.0));
        buffer.set_size(&mut self.font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(
            &mut self.font_system,
            &text,
            glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.queue(TextArea {
            buffer,
            left: 0.0,
            top: 5.0,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    #[profiling::function]
    pub fn update(&mut self, logical_size: (u32, u32), gpu: &mut Gpu) {
        let elements = self.queue.iter().map(|area| glyphon::TextArea {
            buffer: &area.buffer,
            left: area.left,
            top: area.top,
            scale: area.scale,
            bounds: area.bounds,
            default_color: area.default_color,
        });

        let shared = &mut *self.shared.borrow_mut();
        shared.viewport.update(
            &gpu.queue,
            glyphon::Resolution {
                width: logical_size.0,
                height: logical_size.1,
            },
        );

        self.text_renderer
            .prepare(
                &gpu.device,
                &gpu.queue,
                &mut gpu.encoder,
                &mut self.font_system,
                &mut shared.atlas,
                &shared.viewport,
                elements,
                &mut shared.swash_cache,
            )
            .unwrap();

        self.queue.clear();
    }

    pub fn end_frame(&mut self) {
        self.shared.borrow_mut().atlas.trim();
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        let shared = self.shared.borrow();
        self.text_renderer
            .render(&shared.atlas, &shared.viewport, render_pass)
            .unwrap();
    }
}
