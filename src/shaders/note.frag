#version 330

out vec4 fragColor;

in INTERFACE {
  vec2 uv;
  vec2 size;
  float ch;
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

  // Pink & Purple

  // vec3 color = vec3(235.0 / 255.0, 33.0 / 255.0, 136.0 / 255.0);

  // int track = int(mod(In.ch, 2));
  // if (track != 0) {
  //   color = vec3(115.0 / 255.0, 65.0 / 255.0, 166.0 / 255.0);
  // }

  // Syntwave
  vec3 color = vec3(210.0 / 255.0, 89.0 / 255.0, 222.0 / 255.0);

  if (In.isBlack == 1.0) {
    color = vec3(125.0 / 255.0, 69.0 / 255.0, 134.0 / 255.0);
  }

  int track = int(mod(In.ch, 2));
  if (track != 0) {
    if (In.isBlack != 1.0) {
      color = vec3(93.0 / 255.0, 188.0 / 255.0, 255.0 / 255.0);
    } else {
      color = vec3(48.0 / 255.0, 124.0 / 255.0, 255.0 / 255.0);
    }
  }

  // color *= 1.0 - In.isBlack / 2.0;

  fragColor = vec4(color, 1.0);
}