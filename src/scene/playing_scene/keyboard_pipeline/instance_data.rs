use wgpu::vertex_attr_array;

use zerocopy::AsBytes;

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes)]
pub struct KeyInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub is_black: u32,
}
impl KeyInstance {
    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<KeyInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &vertex_attr_array!(1 => Float2,2 => Float2,3 => Uint),
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, PartialEq)]
pub struct KeyStateInstance {
    pub color: [f32; 3],
}
impl KeyStateInstance {
    pub fn vertex_buffer_descriptor<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<KeyStateInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &vertex_attr_array!(4 => Float3),
        }
    }
}
