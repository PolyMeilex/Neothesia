use wgpu::vertex_attr_array;

use zerocopy::AsBytes;

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes)]
pub struct NoteInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub radius: f32,
}
impl NoteInstance {
    pub fn attributes() -> [wgpu::VertexAttributeDescriptor; 4] {
        vertex_attr_array!(1 => Float2,2 => Float2,3 => Float3,4 => Float)
    }
    pub fn desc<'a>(
        attributes: &'a [wgpu::VertexAttributeDescriptor],
    ) -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<NoteInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes,
        }
    }
}
