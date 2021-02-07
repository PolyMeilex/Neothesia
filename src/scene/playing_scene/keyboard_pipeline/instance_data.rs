use wgpu::vertex_attr_array;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct KeyInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub is_black: u32,
}
impl KeyInstance {
    pub fn attributes() -> [wgpu::VertexAttribute; 3] {
        vertex_attr_array!(1 => Float2,2 => Float2,3 => Uint)
    }
    pub fn layout(attributes: &[wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<KeyInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct KeyStateInstance {
    pub color: [f32; 3],
}
impl KeyStateInstance {
    pub fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<KeyStateInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &vertex_attr_array!(4 => Float3),
        }
    }
}
