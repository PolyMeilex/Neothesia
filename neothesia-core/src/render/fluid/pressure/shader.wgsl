struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) texel_size: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.tex_coords;

    var x: f32 = 1.0 / 200.0;
    var y: f32 = 1.0 / 200.0;

    out.texel_size = vec2(x, y);

    return out;
}

@group(0) @binding(0)
var pressure: texture_2d<f32>;
@group(0) @binding(1)
var divergence: texture_2d<f32>;
@group(0) @binding(2)
var s_samper: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = textureSample(pressure, s_samper, in.uv + vec2(0.0, in.texel_size.y)).x;
    let b = textureSample(pressure, s_samper, in.uv - vec2(0.0, in.texel_size.y)).x;
    let r = textureSample(pressure, s_samper, in.uv + vec2(in.texel_size.x, 0.0)).x;
    let l = textureSample(pressure, s_samper, in.uv - vec2(in.texel_size.x, 0.0)).x;

    let divergence = textureSample(divergence, s_samper, in.uv).x;
    let alpha = -1.0;

    let pressure = (l + r + b + t + alpha * divergence) * .25;

    return vec4(pressure, 0.0, 0.0, 1.0);
}
