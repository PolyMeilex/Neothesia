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

    var x: f32 = 1.0 / 200.0;
    var y: f32 = 1.0 / 200.0;

    out.texelSize = vec2(x, y);

    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_samper: sampler;
@group(2) @binding(0)
var t_vel: texture_2d<f32>;

fn bilerp(tex: texture_2d<f32>, uv: vec2<f32>, tsize: vec2<f32>) -> vec4<f32> {
    let st  = uv / tsize - vec2<f32>(0.5, 0.5);
    let iuv = floor(st);
    let fuv = fract(st);

    let a = textureSample(tex, s_samper, (iuv + vec2(0.5, 0.5)) * tsize);
    let b = textureSample(tex, s_samper, (iuv + vec2(1.5, 0.5)) * tsize);
    let c = textureSample(tex, s_samper, (iuv + vec2(0.5, 1.5)) * tsize);
    let d = textureSample(tex, s_samper, (iuv + vec2(1.5, 1.5)) * tsize);

    return mix(mix(a, b, fuv.x), mix(c, d, fuv.x), fuv.y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // var velocity = textureSample(t_vel, s_samper, in.vUv).xy;
    // let velocity = bilerp(t_vel, in.vUv, in.texelSize).xy;

    let velocity = textureSample(t_vel, s_samper, in.vUv).xy;
    let coord = in.vUv - 0.16 * vec2(velocity.x, -velocity.y) * in.texelSize;
    var color = textureSample(t_diffuse, s_samper, coord);
    return vec4(color.rgb, 1.0);
}
