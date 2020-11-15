use wgpu::util::DeviceExt;
use zerocopy::AsBytes;
pub struct Shape {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub indices_len: u32,
}
impl Shape {
    pub fn new(device: &wgpu::Device, vertices: &[Vertex2D], indices: &[u16]) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &vertices.as_bytes(),
            usage: wgpu::BufferUsage::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: &indices.as_bytes(),
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

        #[cfg_attr(rustfmt, rustfmt_skip)] 
        const INDICES: &[u16] = &[     
            0, 1, 2,
            2, 3, 0 
        ];

        Self::new(device, VERTICES, INDICES)
    }

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

        #[cfg_attr(rustfmt, rustfmt_skip)] 
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

        #[cfg_attr(rustfmt, rustfmt_skip)]
        const INDICES: &[u16] = &[
            0, 1, 2,
            0, 2, 3
        ];

        Self::new(device, VERTICES, INDICES)
    }

    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        Vertex2D::desc()
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, AsBytes)]
pub struct Vertex2D {
    position: [f32; 2],
}
impl Vertex2D {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        use std::mem;
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Vertex2D>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float2,
            }],
        }
    }
}
