use wgpu::{
    PipelineLayout, ProgrammableStageDescriptor, RasterizationStateDescriptor,
    RenderPipelineDescriptor, ShaderModule,
};

pub struct RenderPipelineBuilder<'a> {
    render_pipeline_descriptor: RenderPipelineDescriptor<'a>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(layout: &'a PipelineLayout, vertex_module: &'a ShaderModule) -> Self {
        Self {
            render_pipeline_descriptor: RenderPipelineDescriptor {
                layout: &layout,
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: vertex_module,
                    entry_point: "main",
                },
                fragment_stage: None,
                rasterization_state: Some(RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: wgpu::CullMode::None,
                    depth_bias: 0,
                    depth_bias_slope_scale: 0.0,
                    depth_bias_clamp: 0.0,
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            },
        }
    }

    pub fn fragment_stage(mut self, fragment_module: &'a ShaderModule) -> Self {
        self.render_pipeline_descriptor.fragment_stage = Some(ProgrammableStageDescriptor {
            module: fragment_module,
            entry_point: "main",
        });
        self
    }

    pub fn vertex_buffers(mut self, vertex_buffers: &'a [wgpu::VertexBufferDescriptor]) -> Self {
        self.render_pipeline_descriptor.vertex_state.vertex_buffers = vertex_buffers;
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&self.render_pipeline_descriptor)
    }
}
