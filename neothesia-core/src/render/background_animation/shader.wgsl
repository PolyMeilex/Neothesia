struct TimeUniform {
    time: f32,
}

@group(0) @binding(0)
var<uniform> time_uniform: TimeUniform;

struct Vertex {
    @location(0) position: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv_position: vec2<f32>,
}

@vertex
fn vs_main(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv_position = (vertex.position + vec2<f32>(1.0, 1.0)) / 2.0;
    return out;
}


fn rot_z(angle: f32) -> mat2x2<f32> {
    let ca = cos(angle);
    let sa = sin(angle);
    return mat2x2<f32>(
        vec2<f32>(ca, -sa),
        vec2<f32>(sa, ca)
    );
}

fn note_render(uv: vec2<f32>, pos: f32, color: vec3<f32>) -> vec3<f32> {
    let mod_x: f32 = uv.x % (0.1 * 2.5 * 2.0);

    var col: vec3<f32> = vec3<f32>(0.35, 0.08, 0.85);

    if pos == 0.5 {
        col = vec3<f32>(0.16, 0.02, 0.44);
    }

    if uv.y > 0.0 && uv.y < 0.5 {
        return mix(color, col, vec3<f32>(smoothstep(-0.002, 0., 127. / 5800. - abs(mod_x - pos))));
    } else {
        return color;
    }
}

const speed: f32 = -0.5;
const live_time: f32 = 2.6;

fn mod_glsl(x: f32, y: f32) -> f32 {
    return x - y * floor(x / y);
}

@fragment
fn fs_density_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = in.uv_position;

    let d = f32(mod_glsl(
        floor(uv.x * 10.0) + floor(uv.y * 10.0),
        2.0
    ));
    return vec4<f32>(vec3<f32>(d), 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv: vec2<f32> = in.uv_position;
    var color: vec3<f32> = vec3<f32>(0.01);


    return vec4(sin(6.28 * uv.y), sin(6.28 * uv.x), 0.0, 1.0);

    {
      let d = f32(mod_glsl(
          floor(uv.x * 10.0) + floor(uv.y * 10.0),
          2.0
      ));
      return vec4<f32>(vec3<f32>(d), 1.0);
    }

    if uv.y < 0.5 && uv.y > 0.4 && uv.x > 0.3 && uv.x < 0.4 {
      return vec4(1.0, 1.0, 1.0, 1.0);
    } else{
      return vec4(0.0, 0.0, 0.0, 1.0);
    }

    let d = 0.0;

    uv = uv * rot_z(0.7);
    uv.x = uv.x + 1.0;

    uv.x = uv.x * 1.5;
    uv.x = uv.x % 0.5;

        {
        uv.y = uv.y - 1.5;

        var off: f32 = 0.0;
        var pos: vec2<f32> = uv;

        pos.y = pos.y - (((time_uniform.time * speed + off) / 5.0) % 1.0) * live_time;
        color = note_render(pos, 0.1, color);

        off = 1.0;
        pos = uv;
        pos.y = pos.y - (((time_uniform.time * speed + off) / 5.0) % 1.0) * live_time;
        color = note_render(pos, 0.1 * 2.0, color);

        off = 3.0;
        pos = uv;
        pos.y = pos.y - (((time_uniform.time * speed + off) / 5.0) % 1.0) * live_time;
        color = note_render(pos, 0.1 * 3.0, color);

        off = 2.0;
        pos = uv;
        pos.y = pos.y - (((time_uniform.time * speed + off) / 5.0) % 1.0) * live_time;
        color = note_render(pos, 0.1 * 4.0, color);

        off = 0.0;
        pos = uv;
        pos.y = pos.y - (((time_uniform.time * speed + off) / 5.0) % 1.0) * live_time;
        color = note_render(pos, 0.1 * 5.0, color);

        off = 4.0;
        pos = uv;
        pos.y = pos.y - (((time_uniform.time * speed + off) / 5.0) % 1.0) * live_time;
        color = note_render(pos, 0.1 * 5.0, color);
    }


    return vec4<f32>(color / 2.5, 0.5);
}
