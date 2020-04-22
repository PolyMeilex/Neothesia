use super::NoteInstance;

use crate::wgpu_jumpstart::{shader, Gpu, Instances, RenderPipelineBuilder, SimpleQuad, Uniform};

use crate::MainState;

use zerocopy::AsBytes;

pub struct NotesPipeline {
    render_pipeline: wgpu::RenderPipeline,

    simple_quad: SimpleQuad,

    instances: Instances<NoteInstance>,
    time_uniform: Uniform<TimeUniform>,
}

impl<'a> NotesPipeline {
    pub fn new(state: &MainState, gpu: &Gpu, midi: &lib_midi::Midi) -> Self {
        let vs_module = shader::create_module(&gpu.device, include_bytes!("shader/quad.vert.spv"));
        let fs_module = shader::create_module(&gpu.device, include_bytes!("shader/quad.frag.spv"));

        let time_uniform = Uniform::new(
            &gpu.device,
            TimeUniform::default(),
            wgpu::ShaderStage::VERTEX,
        );

        let render_pipeline_layout =
            &gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    bind_group_layouts: &[
                        &state.transform_uniform.bind_group_layout,
                        &time_uniform.bind_group_layout,
                    ],
                });

        let render_pipeline = RenderPipelineBuilder::new(&render_pipeline_layout, &vs_module)
            .fragment_stage(&fs_module)
            .vertex_buffers(&[
                SimpleQuad::vertex_buffer_descriptor(),
                NoteInstance::vertex_buffer_descriptor(),
            ])
            .build(&gpu.device);

        let simple_quad = SimpleQuad::new(&gpu.device);

        let instances = Instances::new(&gpu.device, midi.merged_track.notes.len());

        Self {
            render_pipeline,

            simple_quad,

            instances,

            time_uniform,
        }
    }
    pub fn render(&'a self, state: &'a MainState, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &state.transform_uniform.bind_group, &[]);
        render_pass.set_bind_group(1, &self.time_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, &self.simple_quad.vertex_buffer, 0, 0);
        render_pass.set_vertex_buffer(1, &self.instances.buffer, 0, 0);

        render_pass.set_index_buffer(&self.simple_quad.index_buffer, 0, 0);

        render_pass.draw_indexed(0..SimpleQuad::indices_len(), 0, 0..self.instances.len());
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
#[derive(Clone, Copy, AsBytes)]
struct TimeUniform {
    time: f32,
}
impl Default for TimeUniform {
    fn default() -> Self {
        Self { time: 0.0 }
    }
}
