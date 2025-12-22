mod instance_data;
pub use instance_data::GlowInstance;

pub mod renderer;
pub use renderer::GlowRenderer;

use wgpu_jumpstart::{Gpu, Instances, Shape, TransformUniform, Uniform, wgpu};

pub struct GlowPipeline {
    render_pipeline: wgpu::RenderPipeline,
    quad: Shape,
    instances: Instances<GlowInstance>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    transform_uniform_bind_group: wgpu::BindGroup,
}

impl<'a> GlowPipeline {
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
                    immediate_size: 0,
                });

        let ri_attrs = GlowInstance::attributes();

        let target = wgpu_jumpstart::default_color_target_state(gpu.texture_format);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: Some(&render_pipeline_layout),
                fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
                ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                    &shader,
                    &[Shape::layout(), GlowInstance::layout(&ri_attrs)],
                ))
            });

        let quad = Shape::new_quad(&gpu.device);
        let instances = Instances::new(&gpu.device, 100_000);

        Self {
            render_pipeline,
            quad,
            instances,
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
            transform_uniform_bind_group: transform_uniform.bind_group.clone(),
        }
    }

    pub fn render(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..self.instances.len());
    }

    pub fn clear(&mut self) {
        self.instances.data.clear();
    }

    pub fn instances(&mut self) -> &mut Vec<GlowInstance> {
        &mut self.instances.data
    }

    pub fn prepare(&mut self) {
        self.instances.update(&self.device, &self.queue);
    }
}
