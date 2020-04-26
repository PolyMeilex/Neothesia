use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};

use super::button_pipeline::{ButtonInstance, ButtonPipeline};
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

pub struct Ui<'a> {
    rectangle_pipeline: ButtonPipeline,
    glyph_brush: GlyphBrush<'a, ()>,
    queue: UiQueue,
}

impl<'a> Ui<'a> {
    pub fn new(state: &MainState, gpu: &mut Gpu) -> Self {
        let rectangle_pipeline = ButtonPipeline::new(state, &gpu.device);
        let font: &[u8] = include_bytes!("./Roboto-Regular.ttf");
        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font)
            .expect("Load font")
            .build(&gpu.device, wgpu::TextureFormat::Bgra8Unorm);

        Self {
            rectangle_pipeline,
            glyph_brush,
            queue: UiQueue::new(),
        }
    }
    pub fn queue_rectangle(&mut self, rectangle: ButtonInstance) {
        self.queue.add_rectangle(rectangle);
    }
    pub fn queue_text(&mut self, section: Section) {
        self.glyph_brush.queue(section);
    }
    pub fn resize(&mut self, _state: &crate::MainState, _gpu: &mut Gpu) {}
    fn update(&mut self, gpu: &mut Gpu) {
        self.rectangle_pipeline.update_instance_buffer(
            &mut gpu.encoder,
            &gpu.device,
            self.queue.clear_rectangles(),
        );
    }
    pub fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainOutput) {
        self.update(gpu);
        let encoder = &mut gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Load,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 0.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.rectangle_pipeline.render(state, &mut render_pass);
        }
        self.glyph_brush
            .draw_queued(
                &gpu.device,
                encoder,
                &frame.view,
                state.window_size.0 as u32,
                state.window_size.1 as u32,
            )
            .expect("glyph_brush");
    }
}

struct UiQueue {
    rectangles: Vec<ButtonInstance>,
}

impl UiQueue {
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
        }
    }
    pub fn add_rectangle(&mut self, rectangle: ButtonInstance) {
        self.rectangles.push(rectangle);
    }
    pub fn clear_rectangles(&mut self) -> Vec<ButtonInstance> {
        std::mem::replace(&mut self.rectangles, Vec::new())
    }
}
