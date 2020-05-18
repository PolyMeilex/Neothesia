use wgpu::vertex_attr_array;

use zerocopy::AsBytes;

#[repr(C)]
#[derive(Debug, Copy, Clone, AsBytes, PartialEq)]
pub struct ButtonInstance {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 3],
    pub radius: f32,
    pub is_hovered: u32,
}
impl ButtonInstance {
    pub fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<ButtonInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &vertex_attr_array!(1 => Float2,2 => Float2,3 => Float3,4 => Float, 5 => Uint),
        }
    }
}
