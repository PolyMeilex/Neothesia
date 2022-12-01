use std::time::Duration;

use wgpu_jumpstart::{wgpu, Gpu, RenderPipelineBuilder, Shape, Uniform};

use bytemuck::{Pod, Zeroable};

pub struct BgPipeline {
    render_pipeline: wgpu::RenderPipeline,

    fullscreen_quad: Shape,

    time_uniform: Uniform<TimeUniform>,
}

impl<'a> BgPipeline {
    pub fn new(gpu: &Gpu) -> Self {
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
            wgpu::ShaderStages::FRAGMENT,
        );

        let render_pipeline_layout =
            &gpu.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[&time_uniform.bind_group_layout],
                    push_constant_ranges: &[],
                });

        let render_pipeline =
            RenderPipelineBuilder::new(render_pipeline_layout, "vs_main", &shader)
                .fragment("fs_main", &shader)
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

    pub fn update_time(&mut self, gpu: &mut Gpu, delta: Duration) {
        self.time_uniform.data.time += delta.as_secs_f32();
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
        // Lets move start of the animation a bit
        Self { time: 10.0 }
    }
}
