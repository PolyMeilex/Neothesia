use crate::wgpu_jumpstart::Window;

use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};

use crate::wgpu_jumpstart::{self, Gpu};

pub struct TextRenderer {
    glyph_brush: GlyphBrush<()>,
}

impl TextRenderer {
    pub fn new(gpu: &Gpu) -> Self {
        let font =
            wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("./Roboto-Regular.ttf"))
                .expect("Load font");
        let glyph_brush =
            GlyphBrushBuilder::using_font(font).build(&gpu.device, wgpu_jumpstart::TEXTURE_FORMAT);

        Self { glyph_brush }
    }

    pub fn queue_text(&mut self, section: Section) {
        self.glyph_brush.queue(section);
    }

    pub fn queue_fps(&mut self, fps: i32) {
        let s = format!("FPS: {}", fps);
        let text = vec![wgpu_glyph::Text::new(&s)
            .with_color([1.0, 1.0, 1.0, 1.0])
            .with_scale(20.0)];

        self.queue_text(Section {
            text,
            screen_position: (0.0, 5.0),
            layout: wgpu_glyph::Layout::Wrap {
                line_breaker: Default::default(),
                h_align: wgpu_glyph::HorizontalAlign::Left,
                v_align: wgpu_glyph::VerticalAlign::Top,
            },
            ..Default::default()
        });
    }

    pub fn render(&mut self, window: &Window, gpu: &mut Gpu, view: &wgpu::TextureView) {
        let encoder = &mut gpu.encoder;

        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = window.state.logical_size;
            (width, height)
        };
        self.glyph_brush
            .draw_queued(
                &gpu.device,
                &mut gpu.staging_belt,
                encoder,
                view,
                window_w.round() as u32,
                window_h.round() as u32,
            )
            .expect("glyph_brush");
    }
}
