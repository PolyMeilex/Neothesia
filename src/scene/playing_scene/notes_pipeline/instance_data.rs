use wgpu::vertex_attr_array;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct NoteInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub radius: f32,
}
impl NoteInstance {
    pub fn attributes() -> [wgpu::VertexAttribute; 4] {
        vertex_attr_array!(1 => Float2,2 => Float2,3 => Float3,4 => Float)
    }
    pub fn layout(attributes: &[wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<NoteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes,
        }
    }
}
