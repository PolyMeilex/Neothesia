mod instance_data;

pub use instance_data::{KeyInstance, KeyStateInstance};

use crate::{target::Target, TransformUniform};

use crate::wgpu_jumpstart::{Gpu, Instances, RenderPipelineBuilder, Shape, Uniform};

pub struct KeyboardPipeline {
    render_pipeline: wgpu::RenderPipeline,
    quad: Shape,

    instances: Instances<KeyInstance>,
    instances_state: Instances<KeyStateInstance>,
}

impl<'a> KeyboardPipeline {
    pub fn new(target: &Target) -> Self {
        let vs_module = target
            .gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("shader/quad.vert.spv"));
        let fs_module = target
            .gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("shader/quad.frag.spv"));

        let render_pipeline_layout =
            &target
                .gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&target.transform_uniform.bind_group_layout],
                    push_constant_ranges: &[],
                });
        let ki_attrs = KeyInstance::attributes();

        let render_pipeline =
            RenderPipelineBuilder::new(&render_pipeline_layout, "main", &vs_module)
                .fragment("main", &fs_module)
                .vertex_buffers(&[
                    Shape::layout(),
                    KeyInstance::layout(&ki_attrs),
                    KeyStateInstance::layout(),
                ])
                .build(&target.gpu.device);

        let quad = Shape::new_quad(&target.gpu.device);
        let instances = Instances::new(&target.gpu.device, 88);
        let instances_state = Instances::new(&target.gpu.device, 88);

        Self {
            render_pipeline,
            quad,
            instances,
            instances_state,
        }
    }
    pub fn render(
        &'a self,
        transform_uniform: &'a Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &transform_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));
        render_pass.set_vertex_buffer(2, self.instances_state.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..self.instances.len());
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
