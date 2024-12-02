mod instance_data;

pub use instance_data::NoteInstance;

use wgpu_jumpstart::{wgpu, Gpu, Instances, Shape, TransformUniform, Uniform};

use bytemuck::{Pod, Zeroable};

pub struct WaterfallPipeline {
    render_pipeline: wgpu::RenderPipeline,

    quad: Shape,

    instances: Instances<NoteInstance>,
    time_uniform: Uniform<TimeUniform>,

    text_renderer: TextRenderer,    
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
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[
                        &transform_uniform.bind_group_layout,
                        &time_uniform.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                });

        let ni_attrs = NoteInstance::attributes();

        let target = wgpu_jumpstart::default_color_target_state(gpu.texture_format);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: Some(&render_pipeline_layout),
                fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
                ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                    &shader,
                    &[Shape::layout(), NoteInstance::layout(&ni_attrs)],
                ))
            });

        let quad = Shape::new_quad(&gpu.device);

        let instances = Instances::new(&gpu.device, notes_count);

        let text_renderer = TextRenderer::new(&gpu.device, &gpu.queue);

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

    pub fn add_label(&mut self, text: &str, position: [f32; 2]) {
        self.text_renderer.add_text(text, position);
    
    pub fn clear(&mut self) {
        self.instances.data.clear();
    }

    pub fn instances(&mut self) -> &mut Vec<NoteInstance> {
        &mut self.instances.data
    }

    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.instances.update(device, queue);
    }

    pub fn speed(&self) -> f32 {
        self.time_uniform.data.speed
    }

    pub fn set_speed(&mut self, queue: &wgpu::Queue, speed: f32) {
        self.time_uniform.data.speed = speed;
        self.time_uniform.update(queue);
    }

    pub fn update_time(&mut self, queue: &wgpu::Queue, time: f32) {
        self.time_uniform.data.time = time;
        self.time_uniform.update(queue);
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TimeUniform {
    time: f32,
    speed: f32,
}

impl Default for TimeUniform {
    fn default() -> Self {
        Self {
            time: 0.0,
            speed: 400.0,
        }
    }
}
