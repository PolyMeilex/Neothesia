#version 330

in vec2 pos;

out INTERFACE {
  vec2 uv;
  vec2 size;
}
Out;

uniform vec2 btnPos;
uniform vec2 btnSize;

void main() {
  vec2 posOut = pos;
  posOut *= btnSize;
  posOut.x += btnSize.x;
  posOut.y -= btnSize.y;
  posOut += btnPos;

  Out.uv = btnSize * pos;
  Out.size = btnSize;
  gl_Position = vec4(posOut, 0.0, 1.0);
}