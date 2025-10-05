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
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = view_uniform.transform * vec4<f32>(vertex.position * view_uniform.scale, 0.0, 1.0);
    out.tex_coords = vertex.tex_coords;
    return out;
}

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct QuadUniform {
    pos: vec2<f32>,
    size: vec2<f32>,
    border_radius: vec4<f32>,
}

@group(1) @binding(2)
var<uniform> quad: QuadUniform;

// SFD code is licenced under: MIT by Héctor Ramón & Iced contributors
fn distance_alg(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32
) -> f32 {
    var inner_half_size: vec2<f32> = (size - vec2<f32>(radius, radius) * 2.0) / 2.0;
    var top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    return rounded_box_sdf(frag_coord - top_left - inner_half_size, inner_half_size, 0.0);
}

// Given a vector from a point to the center of a rounded rectangle of the given `size` and
// border `radius`, determines the point's distance from the nearest edge of the rounded rectangle
fn rounded_box_sdf(to_center: vec2<f32>, size: vec2<f32>, radius: f32) -> f32 {
    return length(max(abs(to_center) - size + vec2<f32>(radius, radius), vec2<f32>(0.0, 0.0))) - radius;
}

// Based on the fragment position and the center of the quad, select one of the 4 radii.
// Order matches CSS border radius attribute:
// radii.x = top-left, radii.y = top-right, radii.z = bottom-right, radii.w = bottom-left
fn select_border_radius(radii: vec4<f32>, position: vec2<f32>, center: vec2<f32>) -> f32 {
    var rx = radii.x;
    var ry = radii.y;
    rx = select(radii.x, radii.y, position.x > center.x);
    ry = select(radii.w, radii.z, position.x > center.x);
    rx = select(rx, ry, position.y > center.y);
    return rx;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let quad_border_radius = quad.border_radius * view_uniform.scale;
    let quad_pos = quad.pos * view_uniform.scale;
    let quad_size = quad.size * view_uniform.scale;

    var border_radius = select_border_radius(
        quad_border_radius,
        in.position.xy,
        (quad_pos + (quad_size * 0.5)).xy
    );

    var dist: f32 = distance_alg(
        in.position.xy,
        quad_pos,
        quad_size,
        border_radius,
    );

    var alpha: f32 = 1.0 - smoothstep(
        max(border_radius - 0.5, 0.0),
        border_radius + 0.5,
        dist
    );

    let out = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return vec4(out.xyz, out.w * alpha);
}
