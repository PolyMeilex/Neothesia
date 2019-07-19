#version 330

out vec4 fragColor;

in INTERFACE {
  vec2 uv;
  vec2 size;
}
In;

uniform int btnHover;

void main() {
  float radiusPosition = pow(abs(In.uv.x / In.size.x), In.size.x / 0.005) +
                         pow(abs(In.uv.y / In.size.y), In.size.y / 0.005);

  if (radiusPosition > 1.0) {
    discard;
  }

  vec3 color = vec3(0.08);
  float a = 0.6;

  float y_uv_pos = ((In.uv.y / In.size.y) + 1.0) / 2.0;

  if (btnHover == 1) {
    color = vec3(0.05);
  }

  if (y_uv_pos < 0.07) {
    color = vec3(160.0 / 255.0, 81.0 / 255.0, 238.0 / 255.0);

    if (btnHover == 1) {
      color = vec3(56.0 / 255.0, 145.0 / 255.0, 255.0 / 255.0);
    }

    a = 1.0;
  }

  fragColor = vec4(color, a);
}