mod instance_data;
pub use instance_data::RectangleInstance;

use crate::wgpu_jumpstart::{Gpu, Instances, RenderPipelineBuilder, Shape, Uniform};
use crate::TransformUniform;

pub struct RectanglePipeline {
    render_pipeline: wgpu::RenderPipeline,

    quad: Shape,

    instances: Instances<RectangleInstance>,
}

impl<'a> RectanglePipeline {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        let vs_module = gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("./shader/quad.vert.spv"));
        let fs_module = gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("./shader/quad.frag.spv"));

        // let shader = gpu
        //     .device
        //     .create_shader_module(&wgpu::ShaderModuleDescriptor {
        //         label: None,
        //         source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!(
        //             "./shader/shader.wgsl"
        //         ))),
        //         flags: wgpu::ShaderFlags::all(),
        //     });

        let render_pipeline_layout =
            gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&transform_uniform.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let ri_attrs = RectangleInstance::attributes();

        let render_pipeline =
            RenderPipelineBuilder::new(&render_pipeline_layout, "main", &vs_module)
                .fragment("main", &fs_module)
                .vertex_buffers(&[Shape::layout(), RectangleInstance::layout(&ri_attrs)])
                .build(&gpu.device);

        let quad = Shape::new_quad(&gpu.device);
        let instances = Instances::new(&gpu.device, 100_000);

        Self {
            render_pipeline,

            quad,

            instances,
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

        render_pass.set_index_buffer(self.quad.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

        render_pass.draw_indexed(0..self.quad.indices_len, 0, 0..self.instances.len());
    }
    pub fn update_instance_buffer(
        &mut self,
        command_encoder: &mut wgpu::CommandEncoder,
        device: &wgpu::Device,
        instances: Vec<RectangleInstance>,
    ) {
        if self.instances.data != instances {
            self.instances.data = instances;
            self.instances.update(command_encoder, device);
        }
    }
}
