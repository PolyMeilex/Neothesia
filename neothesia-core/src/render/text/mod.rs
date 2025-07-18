use std::{cell::RefCell, rc::Rc, sync::Arc};

use glyphon::FontSystem;
use wgpu_jumpstart::Gpu;

pub use glyphon;

use crate::utils::Rect;

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
    text_areas: Vec<TextArea>,
    shared: Rc<RefCell<TextShared>>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl TextRendererInstance {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, shared: Rc<RefCell<TextShared>>) -> Self {
        let text_renderer = glyphon::TextRenderer::new(
            &mut shared.borrow_mut().atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

        Self {
            text_renderer,
            text_areas: Vec::new(),
            shared,
            device: device.clone(),
            queue: queue.clone(),
        }
    }

    pub fn queue(&mut self, area: TextArea) {
        self.text_areas.push(area);
    }

    pub fn update(&mut self, logical_size: (u32, u32), font_system: &mut FontSystem) {
        let shared = &mut *self.shared.borrow_mut();
        let elements = self.text_areas.iter().map(|area| glyphon::TextArea {
            buffer: &area.buffer,
            left: area.left,
            top: area.top,
            scale: area.scale,
            bounds: area.bounds,
            default_color: area.default_color,
            custom_glyphs: &[],
        });

        shared.viewport.update(
            &self.queue,
            glyphon::Resolution {
                width: logical_size.0,
                height: logical_size.1,
            },
        );

        self.text_renderer
            .prepare(
                &self.device,
                &self.queue,
                font_system,
                &mut shared.atlas,
                &shared.viewport,
                elements,
                &mut shared.swash_cache,
            )
            .unwrap();

        self.text_areas.clear();
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        let shared = self.shared.borrow();
        self.text_renderer
            .render(&shared.atlas, &shared.viewport, render_pass)
            .unwrap();
    }
}

pub struct TextRenderer {
    font_system: glyphon::FontSystem,
    _cache: glyphon::Cache,

    default_instance: TextRendererInstance,
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
        let atlas = glyphon::TextAtlas::new(&gpu.device, &gpu.queue, &cache, gpu.texture_format);

        let viewport = glyphon::Viewport::new(&gpu.device, &cache);

        let shared = Rc::new(RefCell::new(TextShared {
            viewport,
            atlas,
            swash_cache,
        }));

        let default_instance = TextRendererInstance::new(&gpu.device, &gpu.queue, shared);

        Self {
            font_system,
            _cache: cache,
            default_instance,
        }
    }

    pub fn new_renderer(&mut self) -> TextRendererInstance {
        TextRendererInstance::new(
            &self.default_instance.device,
            &self.default_instance.queue,
            self.default_instance.shared.clone(),
        )
    }

    pub fn font_system(&mut self) -> &mut glyphon::FontSystem {
        &mut self.font_system
    }

    pub fn queue_mut(&mut self) -> &mut Vec<TextArea> {
        &mut self.default_instance.text_areas
    }

    pub fn queue(&mut self, area: TextArea) {
        self.default_instance.queue(area);
    }

    pub fn queue_text(&mut self, text: &str) {
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(15.0, 15.0));
        buffer.set_size(&mut self.font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(
            &mut self.font_system,
            text,
            &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
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
        buffer.set_text(&mut self.font_system, text, &attrs, glyphon::Shaping::Basic);
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

    pub fn queue_buffer_left(&mut self, rect: Rect, buffer: glyphon::Buffer) {
        let (_text_w, text_h) = Self::measure(&buffer);

        let origin = rect.origin;
        let size = rect.size;

        self.queue_buffer(
            origin.x,
            origin.y + size.height / 2.0 - text_h / 2.0,
            buffer,
        );
    }

    pub fn queue_buffer_right(&mut self, rect: Rect, buffer: glyphon::Buffer) {
        let (text_w, text_h) = Self::measure(&buffer);

        let origin = rect.origin;
        let size = rect.size;

        self.queue_buffer(
            origin.x + size.width - text_w,
            origin.y + size.height / 2.0 - text_h / 2.0,
            buffer,
        );
    }

    pub fn queue_buffer_centered(&mut self, rect: Rect, buffer: glyphon::Buffer) {
        let (text_w, text_h) = Self::measure(&buffer);

        let origin = rect.origin;
        let size = rect.size;

        self.queue_buffer(
            origin.x + size.width / 2.0 - text_w / 2.0,
            origin.y + size.height / 2.0 - text_h / 2.0,
            buffer,
        );
    }

    pub fn queue_icon(&mut self, x: f32, y: f32, size: f32, icon: &str) {
        let buffer = self.gen_buffer_with_attr(
            size,
            icon,
            glyphon::Attrs::new().family(glyphon::Family::Name("bootstrap-icons")),
        );

        self.queue_buffer(x, y, buffer);
    }

    pub fn queue_fps(&mut self, fps: f64, y: f32) {
        let text = format!("FPS: {}", fps.round() as u32);
        let mut buffer =
            glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(15.0, 15.0));
        buffer.set_size(&mut self.font_system, Some(f32::MAX), Some(f32::MAX));
        buffer.set_text(
            &mut self.font_system,
            &text,
            &glyphon::Attrs::new().family(glyphon::Family::SansSerif),
            glyphon::Shaping::Basic,
        );
        buffer.shape_until_scroll(&mut self.font_system, false);

        self.queue(TextArea {
            buffer,
            left: 0.0,
            top: y,
            scale: 1.0,
            bounds: glyphon::TextBounds::default(),
            default_color: glyphon::Color::rgb(255, 255, 255),
        });
    }

    #[profiling::function]
    pub fn update(&mut self, logical_size: (u32, u32)) {
        self.default_instance
            .update(logical_size, &mut self.font_system);
    }

    pub fn end_frame(&mut self) {
        self.default_instance.shared.borrow_mut().atlas.trim();
    }

    pub fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>) {
        self.default_instance.render(render_pass);
    }
}
