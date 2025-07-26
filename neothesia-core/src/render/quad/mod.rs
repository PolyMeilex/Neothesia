mod instance_data;
pub use instance_data::QuadInstance;

use wgpu_jumpstart::{wgpu, Gpu, Instances, Shape, TransformUniform, Uniform};

use crate::utils::Rect;

pub struct QuadsLayer {
    scissor_rect: Rect,
    quads: Instances<QuadInstance>,
}

#[derive(Clone)]
pub struct QuadPipeline {
    render_pipeline: wgpu::RenderPipeline,
    transform_uniform_bind_group: wgpu::BindGroup,
    quad: Shape,
}

impl QuadPipeline {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RectanglePipeline::shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "./shader.wgsl"
                ))),
            });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&transform_uniform.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let target = wgpu_jumpstart::default_color_target_state(gpu.texture_format);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: Some(&render_pipeline_layout),
                fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
                ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                    &shader,
                    &[
                        Shape::layout(),
                        QuadInstance::layout(&QuadInstance::attributes()),
                    ],
                ))
            });

        Self {
            render_pipeline,
            transform_uniform_bind_group: transform_uniform.bind_group.clone(),
            quad: Shape::new_quad(&gpu.device),
        }
    }

    pub fn new_layer(device: &wgpu::Device, size: usize) -> QuadsLayer {
        QuadsLayer {
            scissor_rect: Rect::zero(),
            quads: Instances::new(device, size),
        }
    }

    #[profiling::function]
    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, layer: &QuadsLayer) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, layer.quads.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..layer.quads.len());
    }
}

pub struct QuadRenderer {
    pipeline: QuadPipeline,
    layers: Vec<QuadsLayer>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> QuadRenderer {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        Self {
            pipeline: QuadPipeline::new(gpu, transform_uniform),
            layers: Vec::new(),
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn ensure_n_layers(&mut self, count: usize) {
        self.layers
            .resize_with(count, || QuadPipeline::new_layer(&self.device, 100));
    }

    pub fn init_layer(&mut self, size: usize) {
        self.layers
            .push(QuadPipeline::new_layer(&self.device, size));
    }

    #[profiling::function]
    pub fn render(&'a self, layer_id: usize, render_pass: &mut wgpu_jumpstart::RenderPass<'a>) {
        let layer = &self.layers[layer_id];

        let pass_size = render_pass.size();

        if layer.scissor_rect == Rect::zero() {
            render_pass.set_scissor_rect(0, 0, pass_size.width, pass_size.height);
        } else {
            render_pass.set_scissor_rect(
                layer.scissor_rect.origin.x as u32,
                layer.scissor_rect.origin.y as u32,
                layer.scissor_rect.size.width as u32,
                layer.scissor_rect.size.height as u32,
            );
        }

        self.pipeline.render(render_pass, layer);

        // Revert
        if layer.scissor_rect != Rect::zero() {
            render_pass.set_scissor_rect(0, 0, pass_size.width, pass_size.height);
        }
    }

    pub fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.quads.data.clear();
        }
    }

    pub fn layer(&mut self, layer_id: usize) -> &mut Vec<QuadInstance> {
        &mut self.layers[layer_id].quads.data
    }

    pub fn set_scissor_rect(&mut self, layer_id: usize, rect: Rect) {
        self.layers[layer_id].scissor_rect = rect;
    }

    pub fn push(&mut self, layer_id: usize, quad: QuadInstance) {
        self.layers[layer_id].quads.data.push(quad)
    }

    #[profiling::function]
    pub fn prepare(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.quads.update(&self.device, &self.queue);
        }
    }
}
