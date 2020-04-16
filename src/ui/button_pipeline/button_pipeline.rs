use super::ButtonInstance;

use crate::wgpu_jumpstart::{shader, Instances, RenderPipelineBuilder, SimpleQuad, Uniform};

use crate::MainState;

pub struct ButtonPipeline {
    render_pipeline: wgpu::RenderPipeline,

    simple_quad: SimpleQuad,

    instances: Instances<ButtonInstance>,
}

impl<'a> ButtonPipeline {
    pub fn new(state: &MainState, device: &wgpu::Device) -> Self {
        let vs_module = shader::create_module(device, include_bytes!("shader/quad.vert.spv"));
        let fs_module = shader::create_module(device, include_bytes!("shader/quad.frag.spv"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&state.transform_uniform.bind_group_layout],
            });

        let render_pipeline = RenderPipelineBuilder::new(&render_pipeline_layout, &vs_module)
            .fragment_stage(&fs_module)
            .vertex_buffers(&[
                SimpleQuad::vertex_buffer_descriptor(),
                ButtonInstance::desc(),
            ])
            .build(device);

        let simple_quad = SimpleQuad::new(device);
        let instances = Instances::new(device, 100000);

        Self {
            render_pipeline,

            simple_quad,

            instances,
        }
    }
    pub fn render(&'a self, state: &'a MainState, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &state.transform_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.simple_quad.vertex_buffer, 0, 0);
        render_pass.set_vertex_buffer(1, &self.instances.buffer, 0, 0);

        render_pass.set_index_buffer(&self.simple_quad.index_buffer, 0, 0);

        render_pass.draw_indexed(0..SimpleQuad::indices_len(), 0, 0..self.instances.len());
    }
    pub fn update_instance_buffer(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        instances: Vec<ButtonInstance>,
    ) {
        if self.instances.data != instances {
            self.instances.data = instances;
            self.instances.update(command_encoder, device);
        }
    }
}
