use wgpu::vertex_attr_array;
use wgpu_jumpstart::wgpu;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct QuadInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
    pub border_radius: [f32; 4],
}

impl Default for QuadInstance {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [0.0, 0.0],
            color: [0.0, 0.0, 0.0, 1.0],
            border_radius: [0.0; 4],
        }
    }
}

impl QuadInstance {
    pub fn attributes() -> [wgpu::VertexAttribute; 4] {
        vertex_attr_array!(1 => Float32x2, 2 => Float32x2, 3 => Float32x4, 4 => Float32x4)
    }

    pub fn layout(attributes: &[wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<QuadInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes,
        }
    }
}
