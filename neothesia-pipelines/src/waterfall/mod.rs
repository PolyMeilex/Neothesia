mod instance_data;

pub use instance_data::NoteInstance;

use wgpu_jumpstart::{
    wgpu, Gpu, Instances, RenderPipelineBuilder, Shape, TransformUniform, Uniform,
};

use bytemuck::{Pod, Zeroable};

pub struct WaterfallPipeline {
    render_pipeline: wgpu::RenderPipeline,

    quad: Shape,

    instances: Instances<NoteInstance>,
    time_uniform: Uniform<TimeUniform>,
}

impl<'a> WaterfallPipeline {
    pub fn new(
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        notes_count: usize,
    ) -> Self {
        let shader = gpu
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("RectanglePipeline::shader"),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
                    "./shader.wgsl"
                ))),
            });

        let time_uniform = Uniform::new(
            &gpu.device,
            TimeUniform::default(),
            wgpu::ShaderStages::VERTEX,
        );

        let render_pipeline_layout =
            &gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &transform_uniform.bind_group_layout,
                        &time_uniform.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let ni_attrs = NoteInstance::attributes();

        let render_pipeline =
            RenderPipelineBuilder::new(render_pipeline_layout, "vs_main", &shader)
                .fragment("fs_main", &shader)
                .vertex_buffers(&[Shape::layout(), NoteInstance::layout(&ni_attrs)])
                .build(&gpu.device);

        let quad = Shape::new_quad(&gpu.device);

        let instances = Instances::new(&gpu.device, notes_count);

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
        self.instances.update(&gpu.queue);
    }

    pub fn update_time(&mut self, gpu: &mut Gpu, time: f32) {
        self.time_uniform.data.time = time;
        self.time_uniform.update(&gpu.queue);
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
