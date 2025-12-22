mod instance_data;
pub use instance_data::QuadInstance;

use wgpu_jumpstart::{Gpu, Instances, Shape, TransformUniform, Uniform, wgpu};

use crate::utils::Rect;

#[derive(Clone)]
struct QuadPipeline {
    render_pipeline: wgpu::RenderPipeline,
    transform_uniform_bind_group: wgpu::BindGroup,
    quad: Shape,
}

impl QuadPipeline {
    fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
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
                    immediate_size: 0,
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

    #[profiling::function]
    fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        quads: &Instances<QuadInstance>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, quads.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..quads.len());
    }
}

pub struct QuadRendererFactory {
    pipeline: QuadPipeline,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl QuadRendererFactory {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        Self {
            pipeline: QuadPipeline::new(gpu, transform_uniform),
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
        }
    }

    pub fn new_renderer(&self) -> QuadRenderer {
        QuadRenderer {
            pipeline: self.pipeline.clone(),
            scissor_rect: Rect::zero(),
            quads: Instances::new(&self.device, 100),
            device: self.device.clone(),
            queue: self.queue.clone(),
        }
    }
}

pub struct QuadRenderer {
    pipeline: QuadPipeline,
    scissor_rect: Rect<u32>,
    quads: Instances<QuadInstance>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> QuadRenderer {
    #[profiling::function]
    pub fn render(&'a self, render_pass: &mut wgpu_jumpstart::RenderPass<'a>) {
        let pass_size = render_pass.size();
        let scissor_rect = self.scissor_rect;
        let has_scissor_rect = scissor_rect != Rect::zero();

        if has_scissor_rect {
            render_pass.set_scissor_rect(
                scissor_rect.origin.x,
                scissor_rect.origin.y,
                scissor_rect.size.width,
                scissor_rect.size.height,
            );
        } else {
            render_pass.set_scissor_rect(0, 0, pass_size.width, pass_size.height);
        }

        self.pipeline.render(render_pass, &self.quads);

        // Revert
        if has_scissor_rect {
            render_pass.set_scissor_rect(0, 0, pass_size.width, pass_size.height);
        }
    }

    pub fn clear(&mut self) {
        self.quads.data.clear();
    }

    pub fn layer(&mut self) -> &mut Vec<QuadInstance> {
        &mut self.quads.data
    }

    pub fn set_scissor_rect(&mut self, rect: Rect<u32>) {
        self.scissor_rect = rect;
    }

    pub fn push(&mut self, quad: QuadInstance) {
        self.quads.data.push(quad)
    }

    #[profiling::function]
    pub fn prepare(&mut self) {
        self.quads.update(&self.device, &self.queue);
    }
}
