struct ViewUniform {
    transform: mat4x4<f32>,
    size: vec2<f32>,
    scale: f32,
}

@group(0) @binding(0)
var<uniform> view_uniform: ViewUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) vUv: vec2<f32>,
    @location(2) texelSize: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;

    out.vUv = out.tex_coords;

    var x: f32 = 1.0 / 1080.0;
    var y: f32 = 1.0 / 720.0;

    out.texelSize = vec2(x, y);

    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;
@group(2) @binding(0)
var t_vel: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var coord = in.vUv - textureSample(t_vel, s_diffuse, in.vUv).xy * in.texelSize;

    var c = textureSample(t_diffuse, s_diffuse, coord);
    return vec4(c.r, c.g, c.b, 1.0);
}
