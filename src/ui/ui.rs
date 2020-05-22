use wgpu_glyph::{GlyphBrush, GlyphBrushBuilder, Section};

use super::button_pipeline::{ButtonInstance, ButtonPipeline};
use crate::rectangle_pipeline::{RectangleInstance, RectanglePipeline};
use crate::wgpu_jumpstart::Gpu;
use crate::MainState;

pub struct Ui<'a> {
    rectangle_pipeline: RectanglePipeline,
    button_pipeline: ButtonPipeline,
    glyph_brush: GlyphBrush<'a, ()>,
    queue: UiQueue,

    transition_pipeline: RectanglePipeline,
    transition_rect_a: f32,
}

impl<'a> Ui<'a> {
    pub fn new(state: &MainState, gpu: &mut Gpu) -> Self {
        let button_pipeline = ButtonPipeline::new(state, &gpu.device);
        let rectangle_pipeline = RectanglePipeline::new(state, &gpu.device);
        let transition_pipeline = RectanglePipeline::new(state, &gpu.device);
        let font: &[u8] = include_bytes!("./Roboto-Regular.ttf");
        let glyph_brush = GlyphBrushBuilder::using_font_bytes(font)
            .expect("Load font")
            .build(&gpu.device, wgpu::TextureFormat::Bgra8Unorm);

        Self {
            rectangle_pipeline,
            button_pipeline,
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
    pub fn queue_button(&mut self, button: ButtonInstance) {
        self.queue.add_button(button);
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
        self.button_pipeline.update_instance_buffer(
            &mut gpu.encoder,
            &gpu.device,
            self.queue.clear_buttons(),
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
            self.button_pipeline.render(state, &mut render_pass);
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

        // Transition
        if self.transition_rect_a != 0.0 {
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
            self.transition_pipeline.render(state, &mut render_pass);
        }
    }
}

struct UiQueue {
    buttons: Vec<ButtonInstance>,
    rectangles: Vec<RectangleInstance>,
}

impl UiQueue {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
            rectangles: Vec::new(),
        }
    }
    pub fn add_button(&mut self, button: ButtonInstance) {
        self.buttons.push(button);
    }
    pub fn add_rectangle(&mut self, rectangle: RectangleInstance) {
        self.rectangles.push(rectangle);
    }
    pub fn clear_buttons(&mut self) -> Vec<ButtonInstance> {
        std::mem::replace(&mut self.buttons, Vec::new())
    }
    pub fn clear_rectangles(&mut self) -> Vec<RectangleInstance> {
        std::mem::replace(&mut self.rectangles, Vec::new())
    }
}
