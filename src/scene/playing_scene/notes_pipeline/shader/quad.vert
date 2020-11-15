#version 450

layout(location=0) in vec2 a_position;

layout(location=1) in vec2 i_pos;
layout(location=2) in vec2 i_size;
layout(location=3) in vec3 i_color;
layout(location=4) in float i_radius;

layout(location=0) out vec3 o_color;
layout(location=1) out vec2 o_uv;
layout(location=2) out vec2 o_size;
layout(location=3) out float o_radius;

layout(set=0, binding=0) 
uniform Uniforms {
    mat4 u_Transform;
    vec2 u_size;
};
layout(set=1, binding=0) 
uniform Uniforms2 {
    float u_time;
};


#define speed 400.0
void main() {
    vec2 pos = i_pos;
    float start = pos.y;
    pos.y = u_size.y - u_size.y / 5.0;

    vec2 size = i_size;
    size.y = size.y * speed;
    
    pos = pos - vec2(0.0,size.y/2.0);    

    vec2 offset = vec2(0.0, -(start - u_time) * speed);
    

    mat4 i_Transform = mat4(
        vec4(size.x, 0.0, 0.0, 0.0),
        vec4(0.0, size.y, 0.0, 0.0),
        vec4(0.0, 0.0, 1.0, 0.0),
        vec4(pos + offset, 0.0, 1.0)
    );
    
    
    o_color = i_color;
    o_radius = i_radius;
    o_uv = a_position;
    o_size = size;

    gl_Position = u_Transform * i_Transform * vec4(a_position, 0.0, 1.0);
}