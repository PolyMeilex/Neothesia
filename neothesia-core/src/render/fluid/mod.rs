mod texture;

mod divergence;
mod gradient_subtract;
mod pressure;

use divergence::DivergencePipeline;
use gradient_subtract::GradientSubtractPipeline;
use pressure::PressurePipeline;
use wgpu::util::DeviceExt;
use wgpu_jumpstart::{Gpu, TransformUniform, Uniform};

pub struct ImageRenderer {
    copy_pipeline: wgpu::RenderPipeline,
    advect_pipeline: wgpu::RenderPipeline,

    transform_uniform_bind_group: wgpu::BindGroup,
    indices: Indices,
    animation: BgPipeline,

    vertex_buffer: wgpu::Buffer,

    first: bool,

    density_buff: DoubleBuff,
    vel_buff: VelDoubleBuff,

    divergence: DivergencePipeline,
    pressure: PressurePipeline,
    gradient_subtract: GradientSubtractPipeline,
}

impl ImageRenderer {
    pub fn new(gpu: &Gpu, transform_uniform: &Uniform<TransformUniform>) -> Self {
        let animation = BgPipeline::new(gpu);

        let double_buff = DoubleBuff::new(gpu);

        let vel_buff = VelDoubleBuff::new(gpu);

        let device = &gpu.device;
        let format = gpu.texture_format;

        let shader_copy = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let shader_advect = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ShaderAdvect"),
            source: wgpu::ShaderSource::Wgsl(include_str!("advect.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &transform_uniform.bind_group_layout,
                    &vel_buff.texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let target = wgpu_jumpstart::default_color_target_state(format);
        let copy_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("copy_pipeline"),
            layout: Some(&render_pipeline_layout),
            fragment: Some(wgpu_jumpstart::default_fragment(
                &shader_copy,
                &[Some(target)],
            )),
            ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                &shader_copy,
                &[Vertex2D::layout()],
            ))
        });

        let advect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("advect_pipeline"),
            layout: Some(
                &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("AdvectPipelineLayout"),
                    bind_group_layouts: &[
                        &transform_uniform.bind_group_layout,
                        // &double_buff.texture_bind_group_layout,
                        &vel_buff.texture_bind_group_layout,
                        &vel_buff.texture_bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                }),
            ),
            fragment: Some(wgpu_jumpstart::default_fragment(
                &shader_advect,
                &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba32Float,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            )),
            ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                &shader_advect,
                &[Vertex2D::layout()],
            ))
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertex()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            copy_pipeline,
            advect_pipeline,

            transform_uniform_bind_group: transform_uniform.bind_group.clone(),
            indices: Indices::new(device),
            vertex_buffer,

            animation,
            first: true,

            density_buff: double_buff,
            vel_buff,

            divergence: DivergencePipeline::new(gpu),
            pressure: PressurePipeline::new(gpu),
            gradient_subtract: GradientSubtractPipeline::new(gpu),
        }
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>) {
        rpass.set_pipeline(&self.copy_pipeline);
        rpass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);
        rpass.set_bind_group(1, &self.density_buff.curr_bind_group, &[]);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.indices.len, 0, 0..1);
    }

    pub fn post_render(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.first {
            self.first = false;
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fluid: Initial pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.density_buff.curr,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],

                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                self.animation.render_density(&mut rpass);
            }
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("fluid: Initial pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &self.vel_buff.curr,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                        depth_slice: None,
                    })],

                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                self.animation.render(&mut rpass);
            }
        }

        // self.density_buff.flip();
        //
        // {
        //     let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        //         label: Some("fluid: diffuse from prev to curr"),
        //         color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        //             view: &self.density_buff.curr,
        //             resolve_target: None,
        //             ops: wgpu::Operations {
        //                 load: wgpu::LoadOp::Clear(wgpu::Color {
        //                     r: 0.0,
        //                     g: 0.0,
        //                     b: 0.0,
        //                     a: 1.0,
        //                 }),
        //                 store: wgpu::StoreOp::Store,
        //             },
        //             depth_slice: None,
        //         })],
        //
        //         depth_stencil_attachment: None,
        //         timestamp_writes: None,
        //         occlusion_query_set: None,
        //     });
        //     rpass.set_pipeline(&self.diffuse_pipeline);
        //     rpass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);
        //     rpass.set_bind_group(1, &self.density_buff.prev_bind_group, &[]);
        //     rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        //     rpass.set_index_buffer(self.indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
        //     rpass.draw_indexed(0..self.indices.len, 0, 0..1);
        // }

        self.vel_buff.flip();

        // advect velocity
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluid: advect vel from prev to curr"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.vel_buff.curr,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],

                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.advect_pipeline);
            rpass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);
            rpass.set_bind_group(1, &self.vel_buff.prev_bind_group, &[]);
            rpass.set_bind_group(2, &self.vel_buff.prev_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.indices.len, 0, 0..1);
        }

        self.density_buff.flip();

        // advect density
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("fluid: advect from prev to curr"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.density_buff.curr,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],

                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.set_pipeline(&self.advect_pipeline);
            rpass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);
            rpass.set_bind_group(1, &self.density_buff.prev_bind_group, &[]);
            rpass.set_bind_group(2, &self.vel_buff.curr_bind_group, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_index_buffer(self.indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
            rpass.draw_indexed(0..self.indices.len, 0, 0..1);
        }

        {
            self.divergence.render(encoder, &self.vel_buff.curr);
            self.pressure.render(encoder, &self.divergence.texture_view);
            self.gradient_subtract.render(
                encoder,
                &self.pressure.texture_view_curr,
                &self.vel_buff.curr,
                &self.vel_buff.prev,
            );
            self.vel_buff.flip();
        }
    }
}

use bytemuck::{Pod, Zeroable};

use super::BgPipeline;

struct DoubleBuff {
    curr: wgpu::TextureView,
    prev: wgpu::TextureView,

    curr_texture: wgpu::Texture,
    prev_texture: wgpu::Texture,

    curr_bind_group: wgpu::BindGroup,
    prev_bind_group: wgpu::BindGroup,

    texture_bind_group_layout: wgpu::BindGroupLayout,
}

impl DoubleBuff {
    fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        let format = gpu.texture_format;

        let size = wgpu::Extent3d {
            width: 200,
            height: 200,
            depth_or_array_layers: 1,
        };

        let curr_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("curr_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let curr = curr_texture.create_view(&Default::default());

        let prev_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("prev_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let prev = prev_texture.create_view(&Default::default());

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("density_texture_bind_group_layout"),
            });

        let curr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&curr),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let prev_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&prev),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            curr,
            curr_texture,

            prev,
            prev_texture,

            curr_bind_group,
            prev_bind_group,

            texture_bind_group_layout,
        }
    }

    fn flip(&mut self) {
        std::mem::swap(&mut self.curr, &mut self.prev);
        std::mem::swap(&mut self.curr_texture, &mut self.prev_texture);
        std::mem::swap(&mut self.curr_bind_group, &mut self.prev_bind_group);
    }
}

struct VelDoubleBuff {
    curr: wgpu::TextureView,
    prev: wgpu::TextureView,

    curr_texture: wgpu::Texture,
    prev_texture: wgpu::Texture,

    curr_bind_group: wgpu::BindGroup,
    prev_bind_group: wgpu::BindGroup,

    texture_bind_group_layout: wgpu::BindGroupLayout,
}

fn rgba8_to_bgra8(rgba: &[u8]) -> Vec<u8> {
    assert!(
        rgba.len() % 4 == 0,
        "Input buffer must have 4 bytes per pixel"
    );

    let mut bgra = Vec::with_capacity(rgba.len());
    for chunk in rgba.chunks_exact(4) {
        let r = chunk[0];
        let g = chunk[1];
        let b = chunk[2];
        let a = chunk[3];
        bgra.extend_from_slice(&[b, g, r, a]);
    }

    bgra
}

impl VelDoubleBuff {
    fn new(gpu: &Gpu) -> Self {
        let device = &gpu.device;
        // let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let format = gpu.texture_format;

        let size = wgpu::Extent3d {
            width: 200,
            height: 200,
            depth_or_array_layers: 1,
        };

        let curr_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("vel_curr_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let curr = curr_texture.create_view(&Default::default());

        // {
        //     let img =
        //         image::load_from_memory(include_bytes!("../../../../assets/vel.png")).unwrap();
        //     let dimensions = img.dimensions();
        //     let rgba = if format == wgpu::TextureFormat::Bgra8UnormSrgb {
        //         rgba8_to_bgra8(&img.to_rgba8())
        //     } else {
        //         img.to_rgba8().to_vec()
        //     };
        //
        //     gpu.queue.write_texture(
        //         wgpu::TexelCopyTextureInfo {
        //             aspect: wgpu::TextureAspect::All,
        //             texture: &curr_texture,
        //             mip_level: 0,
        //             origin: wgpu::Origin3d::ZERO,
        //         },
        //         &rgba,
        //         wgpu::TexelCopyBufferLayout {
        //             offset: 0,
        //             bytes_per_row: Some(4 * dimensions.0),
        //             rows_per_image: Some(dimensions.1),
        //         },
        //         size,
        //     );
        // }

        let prev_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("vel_prev_texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let prev = prev_texture.create_view(&Default::default());

        // {
        //     let img =
        //         image::load_from_memory(include_bytes!("../../../../assets/vel.png")).unwrap();
        //     let dimensions = img.dimensions();
        //     let rgba = if format == wgpu::TextureFormat::Bgra8UnormSrgb {
        //         rgba8_to_bgra8(&img.to_rgba8())
        //     } else {
        //         img.to_rgba8().to_vec()
        //     };
        //
        //     gpu.queue.write_texture(
        //         wgpu::TexelCopyTextureInfo {
        //             aspect: wgpu::TextureAspect::All,
        //             texture: &prev_texture,
        //             mip_level: 0,
        //             origin: wgpu::Origin3d::ZERO,
        //         },
        //         &rgba,
        //         wgpu::TexelCopyBufferLayout {
        //             offset: 0,
        //             bytes_per_row: Some(4 * dimensions.0),
        //             rows_per_image: Some(dimensions.1),
        //         },
        //         size,
        //     );
        // }

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let prev_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("velocity texture_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let curr_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&curr),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("vel_bind_group_curr"),
        });

        let prev_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&prev),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&prev_sampler),
                },
            ],
            label: Some("vel_bind_group_prev"),
        });

        Self {
            curr,
            curr_texture,

            prev,
            prev_texture,

            curr_bind_group,
            prev_bind_group,

            texture_bind_group_layout,
        }
    }

    fn flip(&mut self) {
        std::mem::swap(&mut self.curr, &mut self.prev);
        std::mem::swap(&mut self.curr_texture, &mut self.prev_texture);
        std::mem::swap(&mut self.curr_bind_group, &mut self.prev_bind_group);
    }
}

fn vertex() -> [Vertex2D; 4] {
    [
        Vertex2D {
            position: [-1.0, -1.0],
            texture_cords: [0.0, 1.0],
        },
        Vertex2D {
            position: [-1.0, 1.0],
            texture_cords: [0.0, 0.0],
        },
        Vertex2D {
            position: [1.0, 1.0],
            texture_cords: [1.0, 0.0],
        },
        Vertex2D {
            position: [1.0, -1.0],
            texture_cords: [1.0, 1.0],
        },
    ]
}

struct Indices {
    buffer: wgpu::Buffer,
    len: u32,
}

impl Indices {
    fn new(device: &wgpu::Device) -> Self {
        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 2,
            0, 2, 3
        ];

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            buffer,
            len: INDICES.len() as u32,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex2D {
    position: [f32; 2],
    texture_cords: [f32; 2],
}

impl Vertex2D {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex2D>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
