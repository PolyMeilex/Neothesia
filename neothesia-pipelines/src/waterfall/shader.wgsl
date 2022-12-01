struct ViewUniform {
    transform: mat4x4<f32>,
    size: vec2<f32>,
}

struct TimeUniform {
    time: f32,
}

@group(0) @binding(0)
var<uniform> view_uniform: ViewUniform;

@group(1) @binding(0)
var<uniform> time_uniform: TimeUniform;

struct Vertex {
    @location(0) position: vec2<f32>,
}

struct NoteInstance {
    @location(1) n_position: vec2<f32>,
    @location(2) size: vec2<f32>,
    @location(3) color: vec3<f32>,
    @location(4) radius: f32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,

    @location(0) src_position: vec2<f32>,
    @location(1) size: vec2<f32>,
    @location(2) color: vec3<f32>,
    @location(3) radius: f32,
    @location(4) note_pos: vec2<f32>,
}

let speed: f32 = 400.0;

@vertex
fn vs_main(vertex: Vertex, note: NoteInstance) -> VertexOutput {
    let size = vec2<f32>(note.size.x, note.size.y * speed);

    let y = view_uniform.size.y - view_uniform.size.y / 5.0 - size.y / 2.0;
    let pos = vec2<f32>(note.n_position.x, y) - vec2<f32>(0.0, size.y / 2.0);

    let offset = vec2<f32>(0.0, -(note.n_position.y - time_uniform.time) * speed);

    let transform = mat4x4<f32>(
        vec4<f32>(size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos + offset, 0.0, 1.0)
    );

    var out: VertexOutput;
    out.position = view_uniform.transform * transform * vec4<f32>(vertex.position, 0.0, 1.0);
    out.note_pos = pos + offset;

    out.src_position = vertex.position;
    out.size = size;
    out.color = note.color;
    out.radius = note.radius;

    return out;
}

fn dist(
    frag_coord: vec2<f32>,
    position: vec2<f32>,
    size: vec2<f32>,
    radius: f32,
) -> f32 {
    let inner_size: vec2<f32> = size - vec2<f32>(radius, radius) * 2.0;
    let top_left: vec2<f32> = position + vec2<f32>(radius, radius);
    let bottom_right: vec2<f32> = top_left + inner_size;

    let top_left_distance: vec2<f32> = top_left - frag_coord;
    let bottom_right_distance: vec2<f32> = frag_coord - bottom_right;

    let dist: vec2<f32> = vec2<f32>(
        max(max(top_left_distance.x, bottom_right_distance.x), 0.0),
        max(max(top_left_distance.y, bottom_right_distance.y), 0.0),
    );

    return sqrt(dist.x * dist.x + dist.y * dist.y);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist: f32 = dist(
        in.position.xy,
        in.note_pos,
        in.size,
        in.radius,
    );

    let alpha: f32 = 1.0 - smoothstep(
        max(in.radius - 0.5, 0.0),
        in.radius + 0.5,
        dist,
    );

    return vec4<f32>(in.color, alpha);
}
