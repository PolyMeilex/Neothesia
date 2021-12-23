struct ViewUniform {
    transform: mat4x4<f32>;
    size: vec2<f32>;
};

struct TimeUniform {
    time: f32;
};

[[group(0), binding(0)]]
var<uniform> view_uniform: ViewUniform;

[[group(1), binding(0)]]
var<uniform> time_uniform: TimeUniform;

struct Vertex {
    [[location(0)]] position: vec2<f32>;
};

struct NoteInstance{
    [[location(1)]] position: vec2<f32>;
    [[location(2)]] size: vec2<f32>;
    [[location(3)]] color: vec3<f32>;
    [[location(4)]] radius: f32;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;

    [[location(0)]] src_position: vec2<f32>;
    [[location(1)]] size: vec2<f32>;
    [[location(2)]] color: vec3<f32>;
    [[location(3)]] radius: f32;

};

let speed: f32 = 400.0;

[[stage(vertex)]]
fn vs_main(vertex: Vertex, note: NoteInstance) -> VertexOutput {
    let size = vec2<f32>(note.size.x, note.size.y * speed);
    
    let y = view_uniform.size.y - view_uniform.size.y / 5.0 - size.y / 2.0;
    let pos = vec2<f32>(note.position.x, y) - vec2<f32>(0.0, size.y / 2.0);
    
    let offset = vec2<f32>(0.0, -(note.position.y - time_uniform.time) * speed);

    let transform = mat4x4<f32>(
        vec4<f32>(size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(pos + offset, 0.0, 1.0)
    );

    var out: VertexOutput;
    out.position = view_uniform.transform * transform * vec4<f32>(vertex.position, 0.0, 1.0);
    
    out.src_position = vertex.position;
    out.size = size;
    out.color = note.color;
    out.radius = note.radius;


    return out;
}

fn corner_alpha(radius: f32, pos: vec2<f32>, cords: vec2<f32>) -> f32{
    let lower = radius - 0.7;
    let upper = radius + 0.7;
    return 1.0 - smoothStep(lower, upper, length(pos - cords));
}

fn fragment_alpha(
    position: vec2<f32>,
    size: vec2<f32>,
    radius: vec4<f32>,
) -> f32 {
    let pos = position * size;
    // Top Left
    let tl = vec2<f32>(radius.x, radius.x);
    // Top Right
    let tr = vec2<f32>(size.x - radius.y, radius.y);
    // Bottom Left
    let bl = vec2<f32>(radius.z, size.y - radius.z);
    // Bottom Right
    let br = vec2<f32>(size.x - radius.w, size.y - radius.w);

    if (pos.x < tl.x && pos.y < tl.y) {
        return corner_alpha(radius.x, pos, tl);
    } else if (pos.x > tr.x && pos.y < tr.y){
        return corner_alpha(radius.y, pos, tr);
    } else if (pos.x < bl.x && pos.y > bl.y){
        return corner_alpha(radius.z, pos, bl);
    } else if (pos.x > br.x && pos.y > br.y){
        return corner_alpha(radius.w, pos, br);
    } else {
        return 1.0;
    }
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) ->  [[location(0)]] vec4<f32> {
    let alpha: f32 = fragment_alpha(
        in.src_position.xy,
        in.size,
        vec4<f32>(in.radius)
    );

    return vec4<f32>(in.color, alpha);
}

