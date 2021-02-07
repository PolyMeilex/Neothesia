#version 450

layout(location=0) in vec2 a_position;

layout(location=0) out vec2 uv;

void main() {
    uv = (a_position+vec2(1.0,1.0)) / 2.0;
    gl_Position = vec4(a_position,0.0,1.0);
}