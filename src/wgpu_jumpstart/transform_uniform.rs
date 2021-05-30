use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct TransformUniform {
    transform: [f32; 16],
    size: [f32; 2],
    // must be aligned to largest member (vec4),
    _padding: [f32; 2],
}
impl Default for TransformUniform {
    fn default() -> Self {
        Self {
            transform: orthographic_projection(1080.0, 720.0),
            size: [1080.0, 720.0],
            _padding: [0.0; 2],
        }
    }
}
impl TransformUniform {
    pub fn update(&mut self, width: f32, height: f32) {
        self.transform = orthographic_projection(width, height);
        self.size = [width, height];
    }
}

fn orthographic_projection(width: f32, height: f32) -> [f32; 16] {
    #[rustfmt::skip]
    let out = [
        2.0 / width, 0.0, 0.0, 0.0,
        0.0, -2.0 / height, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ];

    out
}
