use super::NoteInstance;

use crate::wgpu_jumpstart::{shader, Instances, RenderPipelineBuilder, SimpleQuad, Uniform};

use crate::orthographic::orthographic_projection;

use zerocopy::AsBytes;

pub struct NotesPipeline {
    render_pipeline: wgpu::RenderPipeline,

    simple_quad: SimpleQuad,

    instances: Instances<NoteInstance>,
    transform_uniform: Uniform<TransformUniform>,
    time_uniform: Uniform<TimeUniform>,
}

impl<'a> NotesPipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        let vs_module = shader::create_module(device, include_bytes!("shader/quad.vert.spv"));
        let fs_module = shader::create_module(device, include_bytes!("shader/quad.frag.spv"));

        let transform_uniform = Uniform::new(device, TransformUniform::default());
        let time_uniform = Uniform::new(device, TimeUniform::default());

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[
                    &transform_uniform.bind_group_layout,
                    &time_uniform.bind_group_layout,
                ],
            });

        let render_pipeline = RenderPipelineBuilder::new(&render_pipeline_layout, &vs_module)
            .fragment_stage(&fs_module)
            .vertex_buffers(&[
                SimpleQuad::vertex_buffer_descriptor(),
                NoteInstance::vertex_buffer_descriptor(),
            ])
            .build(device);

        let simple_quad = SimpleQuad::new(device);

        let instances = Instances::new(device);

        Self {
            render_pipeline,

            simple_quad,

            instances,
            transform_uniform,
            time_uniform,
        }
    }
    pub fn render(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.transform_uniform.bind_group, &[]);
        render_pass.set_bind_group(1, &self.time_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.simple_quad.vertex_buffer, 0, 0);
        render_pass.set_vertex_buffer(1, &self.instances.buffer, 0, 0);

        render_pass.set_index_buffer(&self.simple_quad.index_buffer, 0, 0);

        render_pass.draw_indexed(0..SimpleQuad::indices_len(), 0, 0..self.instances.len());
    }
    pub fn update_instance_buffer(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        instances: Vec<NoteInstance>,
    ) {
        self.instances.data = instances;
        self.instances.update(command_encoder, device);
    }
    pub fn resize(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        window_size: (f32, f32),
    ) {
        self.transform_uniform.data.transform =
            orthographic_projection(window_size.0, window_size.1);

        self.transform_uniform.data.w = window_size.0;
        self.transform_uniform.data.h = window_size.1;

        self.transform_uniform.update(command_encoder, device);
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
struct TransformUniform {
    transform: [f32; 16],
    w: f32,
    h: f32,
}
impl Default for TransformUniform {
    fn default() -> Self {
        Self {
            transform: orthographic_projection(1080.0, 720.0),
            w: 1080.0,
            h: 720.0,
        }
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
