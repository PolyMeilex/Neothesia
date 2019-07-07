#version 330

out vec4 fragColor;

uniform int btnHover;

void main() {
  vec3 color = vec3(1.0);
  if (btnHover == 1) {
    color /= 2.0;
  }
  fragColor = vec4(color, 1.0);
}