pub struct RenderPipelineBuilder<'a> {
    render_pipeline_descriptor: wgpu::RenderPipelineDescriptor<'a>,
}

impl<'a> RenderPipelineBuilder<'a> {
    pub fn new(
        layout: &'a wgpu::PipelineLayout,
        entry_point: &'a str,
        vertex_module: &'a wgpu::ShaderModule,
    ) -> Self {
        Self {
            render_pipeline_descriptor: wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(layout),
                vertex: wgpu::VertexState {
                    module: vertex_module,
                    entry_point,
                    buffers: &[],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
            },
        }
    }

    pub fn fragment(
        mut self,
        entry_point: &'a str,
        fragment_module: &'a wgpu::ShaderModule,
    ) -> Self {
        self.render_pipeline_descriptor.fragment = Some(wgpu::FragmentState {
            module: fragment_module,
            entry_point,
            targets: &[wgpu::ColorTargetState {
                format: super::TEXTURE_FORMAT,
                color_blend: wgpu::BlendState {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendState {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
        });
        self
    }

    pub fn vertex_buffers(mut self, vertex_buffers: &'a [wgpu::VertexBufferLayout]) -> Self {
        self.render_pipeline_descriptor.vertex.buffers = vertex_buffers;
        self
    }

    pub fn build(self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&self.render_pipeline_descriptor)
    }
}
