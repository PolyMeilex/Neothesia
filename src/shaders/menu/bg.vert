#version 330

in vec2 pos;

out INTERFACE { vec2 st; }
Out;

void main() {
  gl_Position = vec4(pos, 0.0, 1.0);

  Out.st = pos.xy * 0.5 + 0.5;
}