pub fn default_color_target_state(texture_format: wgpu::TextureFormat) -> wgpu::ColorTargetState {
    wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        }),
        write_mask: wgpu::ColorWrites::ALL,
    }
}

pub trait RenderPipelineBuilder<'a> {
    fn builder(layout: &'a wgpu::PipelineLayout, vertex: wgpu::VertexState<'a>) -> Self;
    fn fragment(
        self,
        entry_point: &'a str,
        fragment_module: &'a wgpu::ShaderModule,
        targets: &'a [Option<wgpu::ColorTargetState>],
    ) -> Self;
    fn create_render_pipeline(&self, device: &wgpu::Device) -> wgpu::RenderPipeline;
}

impl<'a> RenderPipelineBuilder<'a> for wgpu::RenderPipelineDescriptor<'a> {
    fn builder(layout: &'a wgpu::PipelineLayout, vertex: wgpu::VertexState<'a>) -> Self {
        wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex,
            fragment: None,
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        }
    }

    fn fragment(
        mut self,
        entry_point: &'a str,
        fragment_module: &'a wgpu::ShaderModule,
        targets: &'a [Option<wgpu::ColorTargetState>],
    ) -> Self {
        self.fragment = Some(wgpu::FragmentState {
            module: fragment_module,
            entry_point,
            targets,
        });
        self
    }

    fn create_render_pipeline(&self, device: &wgpu::Device) -> wgpu::RenderPipeline {
        device.create_render_pipeline(self)
    }
}
