use wgpu_jumpstart::wgpu;
use bytemuck::{Pod, Zeroable};
use wgpu::vertex_attr_array;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct NoteInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub radius: f32,
    pub note_name: [u8; 4], // Add this line
}

impl NoteInstance {
    pub fn attributes() -> [wgpu::VertexAttribute; 5] { // Update number of attributes
        vertex_attr_array!(
            1 => Float32x2, 
            2 => Float32x2, 
            3 => Float32x3, 
            4 => Float32, 
            5 => Uint32x4 // Add this line for note_name
        )
    }

    pub fn layout(attributes: &[wgpu::VertexAttribute]) -> wgpu::VertexBufferLayout {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<NoteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes,
        }
    }
}
