mod instance_data;
pub use instance_data::QuadInstance;

use wgpu_jumpstart::{wgpu, Gpu, Instances, Shape, TransformUniform, Uniform};

pub struct QuadPipeline {
    render_pipeline: wgpu::RenderPipeline,
    quad: Shape,
    instances: Vec<Instances<QuadInstance>>,
}

impl<'a> QuadPipeline {
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
                    push_constant_ranges: &[],
                });

        let ri_attrs = QuadInstance::attributes();

        let target = wgpu_jumpstart::default_color_target_state(gpu.texture_format);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: Some(&render_pipeline_layout),
                fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
                ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                    &shader,
                    &[Shape::layout(), QuadInstance::layout(&ri_attrs)],
                ))
            });

        let quad = Shape::new_quad(&gpu.device);

        Self {
            render_pipeline,
            quad,
            instances: Vec::new(),
        }
    }

    pub fn init_layer(&mut self, gpu: &Gpu, size: usize) {
        self.instances.push(Instances::new(&gpu.device, size));
    }

    #[profiling::function]
    pub fn render(
        &'a self,
        batch_id: usize,
        transform_uniform: &'a Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        let instances = &self.instances[batch_id];
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &transform_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instances.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..instances.len());
    }

    pub fn clear(&mut self) {
        for instances in self.instances.iter_mut() {
            instances.data.clear();
        }
    }

    pub fn instances(&mut self, batch_id: usize) -> &mut Vec<QuadInstance> {
        &mut self.instances[batch_id].data
    }

    pub fn push(&mut self, batch_id: usize, quad: QuadInstance) {
        self.instances[batch_id].data.push(quad)
    }

    #[profiling::function]
    pub fn prepare(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for instances in self.instances.iter_mut() {
            instances.update(device, queue);
        }
    }
}
