use wgpu::util::DeviceExt;

use bytemuck::{Pod, Zeroable};

pub struct Shape {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices_len: u32,
}
impl Shape {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex2D], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsage::INDEX,
        });
        Self {
            vertex_buffer,
            index_buffer,
            indices_len: indices.len() as u32,
        }
    }

    pub fn new_fullscreen_quad(device: &wgpu::Device) -> Self {
        const VERTICES: &[Vertex2D] = &[
            Vertex2D {
                position: [-1.0, -1.0],
            },
            Vertex2D {
                position: [-1.0, 1.0],
            },
            Vertex2D {
                position: [1.0, 1.0],
            },
            Vertex2D {
                position: [1.0, -1.0],
            },
        ];

        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 2,
            2, 3, 0
        ];

        Self::new(device, VERTICES, INDICES)
    }

    #[allow(dead_code)]
    pub fn new_centered_quad(device: &wgpu::Device) -> Self {
        const VERTICES: &[Vertex2D] = &[
            Vertex2D {
                position: [-0.5, -0.5],
            },
            Vertex2D {
                position: [-0.5, 0.5],
            },
            Vertex2D {
                position: [0.5, 0.5],
            },
            Vertex2D {
                position: [0.5, -0.5],
            },
        ];

        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 2,
            2, 3, 0
        ];

        Self::new(device, VERTICES, INDICES)
    }

    pub fn new_quad(device: &wgpu::Device) -> Self {
        const VERTICES: &[Vertex2D] = &[
            Vertex2D {
                position: [0.0, 0.0],
            },
            Vertex2D {
                position: [1.0, 0.0],
            },
            Vertex2D {
                position: [1.0, 1.0],
            },
            Vertex2D {
                position: [0.0, 1.0],
            },
        ];

        #[rustfmt::skip]
        const INDICES: &[u16] = &[
            0, 1, 2,
            0, 2, 3
        ];

        Self::new(device, VERTICES, INDICES)
    }

    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        Vertex2D::layout()
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex2D {
    position: [f32; 2],
}
impl Vertex2D {
    fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex2D>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float2,
            }],
        }
    }
}
