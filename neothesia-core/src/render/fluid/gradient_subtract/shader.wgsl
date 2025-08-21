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
var velocity: texture_2d<f32>;
@group(0) @binding(2)
var sampler_linear: sampler;
@group(0) @binding(3)
var sampler_nearest: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = textureSample(pressure, sampler_nearest, in.uv + vec2(0.0, in.texel_size.y)).x;
    let b = textureSample(pressure, sampler_nearest, in.uv - vec2(0.0, in.texel_size.y)).x;
    let r = textureSample(pressure, sampler_nearest, in.uv + vec2(in.texel_size.x, 0.0)).x;
    let l = textureSample(pressure, sampler_nearest, in.uv - vec2(in.texel_size.x, 0.0)).x;
    let halfrdx = 0.5;

    var velocity = textureSample(velocity, sampler_linear, in.uv).xy;
    velocity.x -= halfrdx * (r - l);

    velocity.y = -velocity.y;
    velocity.y -= halfrdx * (t - b);
    velocity.y = -velocity.y;

    return vec4(velocity, 0.0, 1.0);
}
