struct ViewUniform {
    transform: mat4x4<f32>,
    size: vec2<f32>,
    scale: f32,
}

@group(0) @binding(0)
var<uniform> view_uniform: ViewUniform;

@group(1) @binding(0)
var atlas: texture_2d<f32>;

@group(1) @binding(1)
var atlas_sampler: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) sprite_position: vec2<f32>,
    @location(2) sprite_size: vec2<f32>,
    @location(3) uv_origin: vec2<f32>,
    @location(4) uv_size: vec2<f32>,
    @location(5) tint: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) tint: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    let logical_position = input.sprite_position + input.position * input.sprite_size;

    var out: VertexOutput;
    out.position = view_uniform.transform
        * vec4<f32>(logical_position * view_uniform.scale, 0.0, 1.0);
    out.uv = input.uv_origin + input.position * input.uv_size;
    out.tint = input.tint;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(atlas, atlas_sampler, input.uv);
    // The atlas is a white mask. Using only its alpha allows every sprite to
    // inherit the MIDI track color without retaining chroma-key remnants.
    return vec4<f32>(input.tint.rgb, input.tint.a * sample.a);
}
