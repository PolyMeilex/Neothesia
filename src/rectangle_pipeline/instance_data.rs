use wgpu::vertex_attr_array;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable, PartialEq)]
pub struct RectangleInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}
impl RectangleInstance {
    pub fn attributes() -> [wgpu::VertexAttribute; 3] {
        vertex_attr_array!(1 => Float2,2 => Float2,3 => Float4)
    }
    pub fn layout(attributes: &[wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<RectangleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes,
        }
    }
}
