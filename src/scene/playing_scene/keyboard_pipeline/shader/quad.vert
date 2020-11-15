#version 450

layout(location=0) in vec2 a_position;

layout(location=1) in vec2 i_pos;
layout(location=2) in vec2 i_size;
layout(location=3) in uint i_is_black;
layout(location=4) in vec3 i_color;

layout(location=0) out vec3 o_color;
layout(location=1) out vec2 o_uv;
layout(location=2) out vec2 o_size;
layout(location=3) out float o_is_black;

layout(set=0, binding=0) 
uniform Uniforms {
    mat4 u_Transform;
    vec2 u_size;
};

void main() {
    o_color = i_color;

    o_size = i_size;
    o_uv = a_position;
    o_is_black = float(i_is_black);

    mat4 i_Transform = mat4(
        vec4(i_size.x, 0.0, 0.0, 0.0),
        vec4(0.0, i_size.y, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(i_pos, 0.0, 1.0)
    );
    
    gl_Position = u_Transform * i_Transform * vec4(a_position, 0.0, 1.0);
}