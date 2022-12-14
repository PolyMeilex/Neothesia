use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};
use wgpu_jumpstart::Gpu;

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

    pub fn glyph_brush(&mut self) -> &mut GlyphBrush<()> {
        &mut self.glyph_brush
    }

    pub fn queue_text(&mut self, section: Section) {
        self.glyph_brush.queue(section);
    }

    pub fn queue_fps(&mut self, fps: f64) {
        let s = format!("FPS: {}", fps.round() as u32);
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

    pub fn render(&mut self, logical_size: (f32, f32), gpu: &mut Gpu, view: &wgpu::TextureView) {
        let encoder = &mut gpu.encoder;

        let (window_w, window_h) = logical_size;

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
