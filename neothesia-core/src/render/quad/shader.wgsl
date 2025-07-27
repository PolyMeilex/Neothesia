struct ViewUniform {
    transform: mat4x4<f32>,
    size: vec2<f32>,
    scale: f32,
}

@group(0) @binding(0)
var<uniform> view_uniform: ViewUniform;

struct Vertex {
    @location(0) position: vec2<f32>,
}

struct QuadInstance {
    @location(1) q_position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) border_radius: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,

    @location(0) quad_size: vec2<f32>,
    @location(1) quad_color: vec4<f32>,
    @location(2) quad_border_radius: vec4<f32>,
    @location(3) quad_position: vec2<f32>,
}

@vertex
fn vs_main(vertex: Vertex, quad: QuadInstance) -> VertexOutput {
    var quad_position = quad.q_position * view_uniform.scale;
    var quad_size = quad.size * view_uniform.scale;

    var i_transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(quad_size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, quad_size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(quad_position, 0.0, 1.0)
    );

    var out: VertexOutput;
    out.position = view_uniform.transform * i_transform * vec4<f32>(vertex.position, 0.0, 1.0);

    out.quad_color = quad.color;
    out.quad_position = quad_position;
    out.quad_size = quad_size;

    var max_border_radius = min(quad.size.x, quad.size.y) * 0.5;
    out.quad_border_radius = vec4(
        min(quad.border_radius.x, max_border_radius),
        min(quad.border_radius.y, max_border_radius),
        min(quad.border_radius.z, max_border_radius),
        min(quad.border_radius.w, max_border_radius)
    ) * view_uniform.scale;

    return out;
}

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
    var border_radius = select_border_radius(
        in.quad_border_radius,
        in.position.xy,
        (in.quad_position + (in.quad_size * 0.5)).xy
    );

    var dist: f32 = distance_alg(
        vec2<f32>(in.position.x, in.position.y),
        in.quad_position,
        in.quad_size,
        border_radius
    );

    var alpha: f32 = 1.0 - smoothstep(
        max(border_radius - 0.5, 0.0),
        border_radius + 0.5,
        dist
    );

    return vec4(in.quad_color.xyz, in.quad_color.w * alpha);
}
