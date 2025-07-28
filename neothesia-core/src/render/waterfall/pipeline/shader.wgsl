struct ViewUniform {
    transform: mat4x4<f32>,
    size: vec2<f32>,
    scale: f32,
}

struct TimeUniform {
    time: f32,
    speed: f32,
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

@vertex
fn vs_main(vertex: Vertex, note: NoteInstance) -> VertexOutput {
    let speed = time_uniform.speed;

    let size = vec2<f32>(note.size.x, note.size.y * abs(speed)) * view_uniform.scale;

    // In an ideal world this should not be hard-coded
    let keyboard_h = view_uniform.size.y / 5.0;
    let keyboard_y = view_uniform.size.y - keyboard_h;

    var pos = vec2<f32>(note.n_position.x * view_uniform.scale, keyboard_y);

    if speed > 0.0 {
        // If notes are falling from top to down, we need to adjust the position,
        // as their start is on bottom of the quad rather than top
        pos.y -= size.y;
    }

    // Offset position by playback time
    pos.y -= (note.n_position.y - time_uniform.time) * speed;

    let transform = mat4x4<f32>(
        vec4<f32>(size.x, 0.0,    0.0, 0.0),
        vec4<f32>(0.0,    size.y, 0.0, 0.0),
        vec4<f32>(0.0,    0.0,    1.0, 0.0),
        vec4<f32>(pos.x,  pos.y,  0.0, 1.0)
    );

    var out: VertexOutput;
    out.position = view_uniform.transform * transform * vec4<f32>(vertex.position, 0.0, 1.0);
    out.note_pos = pos;

    out.src_position = vertex.position;
    out.size = size;
    out.color = note.color;
    out.radius = note.radius * view_uniform.scale;

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
