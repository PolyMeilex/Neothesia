use crate::wgpu_jumpstart::{shader, Instances, RenderPipelineBuilder, SimpleQuad, Uniform};
use zerocopy::AsBytes;

use crate::MainState;

pub struct BgPipeline {
    render_pipeline: wgpu::RenderPipeline,

    simple_quad: SimpleQuad,
    time_uniform: Uniform<TimeUniform>,
}

impl<'a> BgPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let vs_module = shader::create_module(device, include_bytes!("shader/quad.vert.spv"));
        let fs_module = shader::create_module(device, include_bytes!("shader/quad.frag.spv"));

        let time_uniform =
            Uniform::new(device, TimeUniform::default(), wgpu::ShaderStage::FRAGMENT);

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&time_uniform.bind_group_layout],
            });

        let render_pipeline = RenderPipelineBuilder::new(&render_pipeline_layout, &vs_module)
            .fragment_stage(&fs_module)
            .vertex_buffers(&[SimpleQuad::vertex_buffer_descriptor()])
            .build(device);

        let simple_quad = SimpleQuad::new(device);

        Self {
            render_pipeline,

            simple_quad,

            time_uniform,
        }
    }
    pub fn render(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.time_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.simple_quad.vertex_buffer, 0, 0);

        render_pass.set_index_buffer(&self.simple_quad.index_buffer, 0, 0);

        render_pass.draw_indexed(0..SimpleQuad::indices_len(), 0, 0..1);
    }
    pub fn update_time(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        time: f32,
    ) {
        self.time_uniform.data.time = time;
        self.time_uniform.update(command_encoder, device);
    }
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes)]
struct TimeUniform {
    time: f32,
}
impl Default for TimeUniform {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}
