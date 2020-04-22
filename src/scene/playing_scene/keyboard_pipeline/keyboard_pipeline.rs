use super::{KeyInstance, KeyStateInstance};

use crate::wgpu_jumpstart::{shader, Gpu, Instances, RenderPipelineBuilder, SimpleQuad, Uniform};

use crate::MainState;

pub struct KeyboardPipeline {
    render_pipeline: wgpu::RenderPipeline,
    simple_quad: SimpleQuad,

    instances: Instances<KeyInstance>,
    instances_state: Instances<KeyStateInstance>,
}

impl<'a> KeyboardPipeline {
    pub fn new(state: &MainState, gpu: &Gpu) -> Self {
        let vs_module = shader::create_module(&gpu.device, include_bytes!("shader/quad.vert.spv"));
        let fs_module = shader::create_module(&gpu.device, include_bytes!("shader/quad.frag.spv"));

        let render_pipeline_layout =
            &gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[&state.transform_uniform.bind_group_layout],
                });

        let render_pipeline = RenderPipelineBuilder::new(&render_pipeline_layout, &vs_module)
            .fragment_stage(&fs_module)
            .vertex_buffers(&[
                SimpleQuad::vertex_buffer_descriptor(),
                KeyInstance::vertex_buffer_descriptor(),
                KeyStateInstance::vertex_buffer_descriptor(),
            ])
            .build(&gpu.device);

        let simple_quad = SimpleQuad::new(&gpu.device);
        let instances = Instances::new(&gpu.device, 88);
        let instances_state = Instances::new(&gpu.device, 88);

        Self {
            render_pipeline,
            simple_quad,
            instances,
            instances_state,
        }
    }
    pub fn render(&'a self, state: &'a MainState, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &state.transform_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.simple_quad.vertex_buffer, 0, 0);
        render_pass.set_vertex_buffer(1, &self.instances.buffer, 0, 0);
        render_pass.set_vertex_buffer(2, &self.instances_state.buffer, 0, 0);

        render_pass.set_index_buffer(&self.simple_quad.index_buffer, 0, 0);

        render_pass.draw_indexed(0..SimpleQuad::indices_len(), 0, 0..self.instances.len());
    }
    pub fn update_instance_buffer(&mut self, gpu: &mut Gpu, instances: Vec<KeyInstance>) {
        self.instances.data = instances;
        self.instances.update(&mut gpu.encoder, &gpu.device);
    }
    pub fn update_notes_state(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        instances_state: Vec<KeyStateInstance>,
    ) {
        if self.instances_state.data != instances_state {
            self.instances_state.data = instances_state;
            self.instances_state.update(command_encoder, device);
        }
    }
}
