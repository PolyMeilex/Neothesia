pub fn orthographic_projection(width: f32, height: f32) -> [f32; 16] {
    #[cfg_attr(rustfmt, rustfmt_skip)]
    [
        2.0 / width, 0.0, 0.0, 0.0,
        0.0, -2.0 / height, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        -1.0, 1.0, 0.0, 1.0,
    ]
}

