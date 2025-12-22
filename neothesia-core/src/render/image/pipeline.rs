use bytes::Bytes;
use neothesia_image::ImageIdentifier;
use wgpu::util::DeviceExt;
use wgpu_jumpstart::{TransformUniform, Uniform};

use crate::Rect;

use super::texture;

pub struct ImageRenderer {
    pipeline: wgpu::RenderPipeline,
    transform_uniform_bind_group: wgpu::BindGroup,
    indices: Indices,
}

impl ImageRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        transform_uniform: &Uniform<TransformUniform>,
    ) -> Self {
        let texture_bind_group_layout = texture_bind_group_layout(device);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &transform_uniform.bind_group_layout,
                    &texture_bind_group_layout,
                ],
                immediate_size: 0,
            });

        let target = wgpu_jumpstart::default_color_target_state(format);

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: Some(&render_pipeline_layout),
            fragment: Some(wgpu_jumpstart::default_fragment(&shader, &[Some(target)])),
            ..wgpu_jumpstart::default_render_pipeline(wgpu_jumpstart::default_vertex(
                &shader,
                &[Vertex2D::layout()],
            ))
        });

        Self {
            pipeline,
            transform_uniform_bind_group: transform_uniform.bind_group.clone(),
            indices: Indices::new(device),
        }
    }

    pub fn render<'rpass>(&'rpass self, rpass: &mut wgpu::RenderPass<'rpass>, image: &Image) {
        rpass.set_pipeline(&self.pipeline);
        rpass.set_bind_group(0, &self.transform_uniform_bind_group, &[]);
        rpass.set_bind_group(1, &image.diffuse_bind_group, &[]);
        rpass.set_vertex_buffer(0, image.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.indices.buffer.slice(..), wgpu::IndexFormat::Uint16);
        rpass.draw_indexed(0..self.indices.len, 0, 0..1);
    }
}

fn texture_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<QuadUniform>() as u64
                    ),
                },
                count: None,
            },
        ],
        label: Some("texture_bind_group_layout"),
    })
}

use bytemuck::{Pod, Zeroable};

#[derive(Clone)]
pub struct Image {
    vertex_buffer: wgpu::Buffer,
    quad_buffer: wgpu::Buffer,
    diffuse_bind_group: wgpu::BindGroup,
    queue: wgpu::Queue,
    rect: Rect,
    bytes: Bytes,
    border_radius: [f32; 4],
}

impl Image {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, bytes: Bytes) -> Self {
        let diffuse_texture = texture::Texture::from_bytes(device, queue, &bytes).unwrap();

        let texture_bind_group_layout = texture_bind_group_layout(device);

        let quad_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&[QuadUniform {
                pos: [0.0, 0.0],
                size: [1.0, 1.0],
                border_radius: [0.0, 0.0, 0.0, 0.0],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &quad_buffer,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&Self::vertex(0.0, 0.0, 1.0, 1.0)),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            vertex_buffer,
            quad_buffer,
            diffuse_bind_group,
            rect: Rect::new((0.0, 0.0).into(), (1.0, 1.0).into()),
            queue: queue.clone(),
            bytes,
            border_radius: [0.0; 4],
        }
    }

    fn vertex(x: f32, y: f32, w: f32, h: f32) -> [Vertex2D; 4] {
        [
            Vertex2D {
                position: [x, y],
                texture_cords: [0.0, 0.0],
            },
            Vertex2D {
                position: [x + w, y],
                texture_cords: [1.0, 0.0],
            },
            Vertex2D {
                position: [x + w, y + h],
                texture_cords: [1.0, 1.0],
            },
            Vertex2D {
                position: [x, y + h],
                texture_cords: [0.0, 1.0],
            },
        ]
    }

    pub fn set_rect(&mut self, rect: Rect, border_radius: [f32; 4]) {
        if self.rect == rect && self.border_radius == border_radius {
            return;
        }

        // TODO: No longer needed, as we have QuadUniform
        let vertex = Self::vertex(rect.min_x(), rect.min_y(), rect.width(), rect.height());
        self.queue
            .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex));

        let quad = QuadUniform {
            pos: [rect.min_x(), rect.min_y()],
            size: [rect.width(), rect.height()],
            border_radius,
        };

        self.queue
            .write_buffer(&self.quad_buffer, 0, bytemuck::cast_slice(&[quad]));

        self.rect = rect;
        self.border_radius = border_radius;
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn identifier(&self) -> ImageIdentifier {
        ImageIdentifier::from_bytes_ptr(&self.bytes)
    }
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
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct QuadUniform {
    pos: [f32; 2],
    size: [f32; 2],
    border_radius: [f32; 4],
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
