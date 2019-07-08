#version 330

out vec4 fragColor;

in INTERFACE {
  vec2 uv;
  vec2 size;
}
In;

uniform int btnHover;

void main() {
  float radiusPosition = pow(abs(In.uv.x / In.size.x), In.size.x / 0.004) +
                         pow(abs(In.uv.y / In.size.y), In.size.y / 0.004);

  if (radiusPosition > 1.0) {
    discard;
  }

  vec3 color = vec3(1.0);
  if (btnHover == 1) {
    color /= 2.0;
  }
  fragColor = vec4(color, 1.0);
}