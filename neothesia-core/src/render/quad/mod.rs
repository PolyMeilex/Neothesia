mod instance_data;
pub use instance_data::QuadInstance;

use wgpu_jumpstart::{wgpu, Gpu, Instances, Shape, TransformUniform, Uniform};

type QuadsLayer = Instances<QuadInstance>;

#[derive(Clone)]
pub struct QuadPipeline {
    render_pipeline: wgpu::RenderPipeline,
    quad: Shape,
}

impl QuadPipeline {
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

        let target = wgpu_jumpstart::default_color_target_state(gpu.texture_format);

        let render_pipeline = gpu
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                layout: Some(&render_pipeline_layout),
                fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
                ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                    &shader,
                    &[
                        Shape::layout(),
                        QuadInstance::layout(&QuadInstance::attributes()),
                    ],
                ))
            });

        Self {
            render_pipeline,
            quad: Shape::new_quad(&gpu.device),
        }
    }

    pub fn new_layer(device: &wgpu::Device, size: usize) -> QuadsLayer {
        Instances::new(device, size)
    }

    #[profiling::function]
    pub fn render<'a>(
        &'a self,
        transform_uniform: &'a Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'a>,
        instances: &QuadsLayer,
    ) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &transform_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.quad.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instances.buffer.slice(..));

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..instances.len());
    }
}

pub struct QuadRenderer {
    pipeline: QuadPipeline,
    layers: Vec<QuadsLayer>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl<'a> QuadRenderer {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        Self {
            pipeline: QuadPipeline::new(gpu, transform_uniform),
            layers: Vec::new(),
            device: gpu.device.clone(),
            queue: gpu.queue.clone(),
        }
    }

    pub fn init_layer(&mut self, size: usize) {
        self.layers
            .push(QuadPipeline::new_layer(&self.device, size));
    }

    #[profiling::function]
    pub fn render(
        &'a self,
        layer_id: usize,
        transform_uniform: &'a Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'a>,
    ) {
        self.pipeline
            .render(transform_uniform, render_pass, &self.layers[layer_id]);
    }

    pub fn clear(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.data.clear();
        }
    }

    pub fn layer(&mut self, layer_id: usize) -> &mut Vec<QuadInstance> {
        &mut self.layers[layer_id].data
    }

    pub fn push(&mut self, layer_id: usize, quad: QuadInstance) {
        self.layers[layer_id].data.push(quad)
    }

    #[profiling::function]
    pub fn prepare(&mut self) {
        for layer in self.layers.iter_mut() {
            layer.update(&self.device, &self.queue);
        }
    }
}
