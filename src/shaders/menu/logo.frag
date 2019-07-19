#version 140
uniform sampler2D tex;
in vec2 v_tex_coords;
out vec4 color;
void main() { color = texture(tex, v_tex_coords); }