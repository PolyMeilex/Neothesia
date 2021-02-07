[[block]]
struct ViewUniform {
    transform: mat4x4<f32>;
    size: vec2<f32>;
};

// 
// Vertex shader
// 

[[location(0)]] var<in> vertex_position: vec2<f32>;
[[location(1)]] var<in> instance_position: vec2<f32>;
[[location(2)]] var<in> instance_size: vec2<f32>;
[[location(3)]] var<in> instance_color: vec4<f32>;

[[builtin(position)]] var<out> out_position: vec4<f32>;
[[location(0)]] var<out> out_color: vec4<f32>;

[[group(0), binding(0)]]
var<uniform> view_uniform: ViewUniform;

[[stage(vertex)]]
fn vs_main() {
    var i_transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(instance_size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, instance_size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(instance_position, 0.0, 1.0)
    );


    out_color = instance_color;
    out_position = view_uniform.transform * i_transform * vec4<f32>(vertex_position, 0.0, 1.0);
}

// 
// Fragment shader
// 

[[location(0)]] var<in> in_color: vec4<f32>;
[[location(0)]] var<out> out_color: vec4<f32>;


[[stage(fragment)]]
fn fs_main() {
    out_color = in_color;
}

