struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;

    return out;
}

fn mod_glsl(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

@fragment
fn fs_main(@location(0) in: VertexOutput) -> @location(0) vec4<f32> {
    let d = f32(mod_glsl(
        floor(in.tex_coords.x * 10.0) + floor(in.tex_coords.y * 10.0),
        2.0
    ));
    return vec4<f32>(vec3<f32>(d), 1.0);
}
