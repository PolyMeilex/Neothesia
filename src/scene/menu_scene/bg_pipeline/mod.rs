use crate::wgpu_jumpstart::{Gpu, RenderPipelineBuilder, Shape, Uniform};

use bytemuck::{Pod, Zeroable};

pub struct BgPipeline {
    render_pipeline: wgpu::RenderPipeline,

    fullscreen_quad: Shape,

    time_uniform: Uniform<TimeUniform>,
}

impl<'a> BgPipeline {
    pub fn new(gpu: &Gpu) -> Self {
        let vs_module = gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("./shader/bg.vert.spv"));
        let fs_module = gpu
            .device
            .create_shader_module(&wgpu::include_spirv!("./shader/bg.frag.spv"));

        let time_uniform = Uniform::new(
            &gpu.device,
            TimeUniform::default(),
            wgpu::ShaderStage::FRAGMENT,
        );

        let render_pipeline_layout =
            &gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&time_uniform.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            RenderPipelineBuilder::new(&render_pipeline_layout, "main", &vs_module)
                .fragment("main", &fs_module)
                .vertex_buffers(&[Shape::layout()])
                .build(&gpu.device);

        let fullscreen_quad = Shape::new_fullscreen_quad(&gpu.device);

        Self {
            render_pipeline,

            fullscreen_quad,

            time_uniform,
        }
    }
    pub fn render(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.time_uniform.bind_group, &[]);

        render_pass.set_vertex_buffer(0, self.fullscreen_quad.vertex_buffer.slice(..));

        render_pass.set_index_buffer(
            self.fullscreen_quad.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );

        render_pass.draw_indexed(0..self.fullscreen_quad.indices_len, 0, 0..1);
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
