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

struct Config {
    color: vec3<f32>,
    point: vec2<f32>,
    radius: f32,
}

@group(0) @binding(0)
var src: texture_2d<f32>;
@group(0) @binding(1)
var s_samper: sampler;
@group(0) @binding(2)
var<uniform> config: Config;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.uv - config.point.xy;
    let splat = exp(-dot(p, p) / config.radius) * config.color;
    let base = textureSample(src, s_samper, in.uv).xyz;
    return vec4(base + splat, 1.0);
}
