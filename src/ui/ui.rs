use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};

use crate::rectangle_pipeline::{RectangleInstance, RectanglePipeline};
use crate::wgpu_jumpstart::{self, Gpu};
use crate::MainState;

pub struct Ui {
    rectangle_pipeline: RectanglePipeline,
    glyph_brush: GlyphBrush<()>,
    queue: UiQueue,

    transition_pipeline: RectanglePipeline,
    transition_rect_a: f32,
}

impl Ui {
    pub fn new(state: &MainState, gpu: &mut Gpu) -> Self {
        let rectangle_pipeline = RectanglePipeline::new(state, &gpu.device);
        let transition_pipeline = RectanglePipeline::new(state, &gpu.device);
        let font =
            wgpu_glyph::ab_glyph::FontArc::try_from_slice(include_bytes!("./Roboto-Regular.ttf"))
                .expect("Load font");
        let glyph_brush =
            GlyphBrushBuilder::using_font(font).build(&gpu.device, wgpu_jumpstart::TEXTURE_FORMAT);

        Self {
            rectangle_pipeline,
            glyph_brush,
            queue: UiQueue::new(),

            transition_rect_a: 0.0,
            transition_pipeline,
        }
    }
    pub fn set_transition_alpha(&mut self, gpu: &mut Gpu, rectangle: RectangleInstance) {
        self.transition_rect_a = rectangle.color[3];
        self.transition_pipeline.update_instance_buffer(
            &mut gpu.encoder,
            &gpu.device,
            vec![rectangle],
        );
    }
    pub fn queue_rectangle(&mut self, rectangle: RectangleInstance) {
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
    pub fn render(&mut self, state: &mut MainState, gpu: &mut Gpu, frame: &wgpu::SwapChainFrame) {
        self.update(gpu);
        let encoder = &mut gpu.encoder;
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.rectangle_pipeline.render(state, &mut render_pass);
        }

        let (window_w, window_h) = {
            let winit::dpi::LogicalSize { width, height } = state.window.state.logical_size;
            (width, height)
        };
        self.glyph_brush
            .draw_queued(
                &gpu.device,
                &mut gpu.staging_belt,
                encoder,
                &frame.output.view,
                window_w.round() as u32,
                window_h.round() as u32,
            )
            .expect("glyph_brush");

        // Transition
        if self.transition_rect_a != 0.0 {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.output.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            self.transition_pipeline.render(state, &mut render_pass);
        }
    }
}

struct UiQueue {
    rectangles: Vec<RectangleInstance>,
}

impl UiQueue {
    pub fn new() -> Self {
        Self {
            rectangles: Vec::new(),
        }
    }
    pub fn add_rectangle(&mut self, rectangle: RectangleInstance) {
        self.rectangles.push(rectangle);
    }
    pub fn clear_rectangles(&mut self) -> Vec<RectangleInstance> {
        std::mem::replace(&mut self.rectangles, Vec::new())
    }
}
