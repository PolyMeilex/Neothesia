mod instance_data;

pub use instance_data::NoteInstance;

use crate::wgpu_jumpstart::{Gpu, Instances, RenderPipelineBuilder, Shape, Uniform};
use crate::{target::Target, TransformUniform};

use bytemuck::{Pod, Zeroable};

pub struct NotesPipeline {
    render_pipeline: wgpu::RenderPipeline,

    quad: Shape,

    instances: Instances<NoteInstance>,
    time_uniform: Uniform<TimeUniform>,
}

impl<'a> NotesPipeline {
    pub fn new(target: &Target, midi: &lib_midi::Midi) -> Self {
        let vs_module = target
            .gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("shader/quad.vert.spv"));
        let fs_module = target
            .gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("shader/quad.frag.spv"));

        let time_uniform = Uniform::new(
            &target.gpu.device,
            TimeUniform::default(),
            wgpu::ShaderStage::VERTEX,
        );

        let render_pipeline_layout =
            &target
                .gpu
                .device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &target.transform_uniform.bind_group_layout,
                        &time_uniform.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let ni_attrs = NoteInstance::attributes();

        let render_pipeline =
            RenderPipelineBuilder::new(&render_pipeline_layout, "main", &vs_module)
                .fragment("main", &fs_module)
                .vertex_buffers(&[Shape::layout(), NoteInstance::layout(&ni_attrs)])
                .build(&target.gpu.device);

        let quad = Shape::new_quad(&target.gpu.device);

        let instances = Instances::new(&target.gpu.device, midi.merged_track.notes.len());

        Self {
            render_pipeline,

            quad,

            instances,

            time_uniform,
        }
    }
    pub fn render(
        &'a self,
        transform_uniform: &'a Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &transform_uniform.bind_group, &[]);
        render_pass.set_bind_group(1, &self.time_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instances.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..self.instances.len());
    }
    pub fn update_instance_buffer(&mut self, gpu: &mut Gpu, instances: Vec<NoteInstance>) {
        self.instances.data = instances;
        self.instances.update(&mut gpu.encoder, &gpu.device);
    }
    pub fn update_time(&mut self, gpu: &mut Gpu, time: f32) {
        self.time_uniform.data.time = time;
        self.time_uniform.update(&mut gpu.encoder, &gpu.device);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TimeUniform {
    time: f32,
}
impl Default for TimeUniform {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}
