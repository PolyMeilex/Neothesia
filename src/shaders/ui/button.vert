#version 330

in vec2 pos;

uniform vec2 btnPos;
uniform vec2 btnSize;

void main() {
  vec2 posOut = pos;
  posOut *= btnSize;
  posOut.x += btnSize.x;
  posOut.y -= btnSize.y;
  posOut += btnPos;

  gl_Position = vec4(posOut, 0.0, 1.0);
}