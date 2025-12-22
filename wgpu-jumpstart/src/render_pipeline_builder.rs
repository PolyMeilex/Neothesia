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

pub fn default_fragment<'a>(
    module: &'a wgpu::ShaderModule,
    targets: &'a [Option<wgpu::ColorTargetState>],
) -> wgpu::FragmentState<'a> {
    wgpu::FragmentState {
        module,
        entry_point: Some("fs_main"),
        targets,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    }
}

pub fn default_vertex<'a>(
    module: &'a wgpu::ShaderModule,
    buffers: &'a [wgpu::VertexBufferLayout<'a>],
) -> wgpu::VertexState<'a> {
    wgpu::VertexState {
        module,
        entry_point: Some("vs_main"),
        buffers,
        compilation_options: wgpu::PipelineCompilationOptions::default(),
    }
}

pub fn default_render_pipeline(vertex: wgpu::VertexState) -> wgpu::RenderPipelineDescriptor {
    wgpu::RenderPipelineDescriptor {
        label: None,
        layout: None,
        vertex,
        fragment: None,
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    }
}
