#version 330

out vec4 fragColor;

in INTERFACE {
  vec2 uv;
  vec2 size;
  float isBlack;
}
In;

uniform float time;

void main() {
  float radiusPosition =
      pow(abs(In.uv.x / (0.5 * In.size.x)), In.size.x / 0.01) +
      pow(abs(In.uv.y / (0.5 * In.size.y)), In.size.y / 0.01);

  if (radiusPosition > 1.0) {
    discard;
  }

  vec2 st = (In.uv + 1.0) / 2.0;
  // vec3 color = vec3(0.4, (sin(time / 10.0)), 0.7);
  vec3 color = vec3(115.0 / 255.0, 65.0 / 255.0, 166.0 / 255.0);

  color *= 1.0 - In.isBlack / 3.0;

  fragColor = vec4(color, 1.0);
}