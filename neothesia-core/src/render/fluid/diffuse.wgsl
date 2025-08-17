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
    @location(2) vL: vec2<f32>,
    @location(3) vR: vec2<f32>,
    @location(4) vT: vec2<f32>,
    @location(5) vB: vec2<f32>,
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

    out.vL = out.vUv - vec2(x, 0.0);
    out.vR = out.vUv + vec2(x, 0.0);
    out.vT = out.vUv + vec2(0.0, y);
    out.vB = out.vUv - vec2(0.0, y);

    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var c = textureSample(t_diffuse, s_diffuse, in.vUv).r;

    var r = textureSample(t_diffuse, s_diffuse, in.vR).r;
    var l = textureSample(t_diffuse, s_diffuse, in.vL).r;
    var t = textureSample(t_diffuse, s_diffuse, in.vT).r;
    var b = textureSample(t_diffuse, s_diffuse, in.vB).r;

    var a = 1.0;
    var v = (c + a * (l + r + t + b)) / (1.0 + 4.0 * a);

    return vec4(v, v, v, 1.0);
}
