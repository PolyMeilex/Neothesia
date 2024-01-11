struct ViewUniform {
    transform: mat4x4<f32>,
    size: vec2<f32>,
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
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) quad_color: vec4<f32>,
}

@vertex
fn vs_main(vertex: Vertex, quad: QuadInstance) -> VertexOutput {
    var i_transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(quad.size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, quad.size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(quad.q_position, 0.0, 1.0)
    );

    var out: VertexOutput;
    out.position = view_uniform.transform * i_transform * vec4<f32>(vertex.position, 0.0, 1.0);
    out.uv = vertex.position;
    out.quad_color = quad.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let m = distance(in.uv, vec2(0.5, 0.5)) * 2.0;
    return mix(in.quad_color, vec4<f32>(0.0), m); 

}
