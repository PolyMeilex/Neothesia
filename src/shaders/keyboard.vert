#version 330

in vec2 pos;

#define notesCount 52.0

out INTERFACE { vec2 uv; }
Out;

void main() {
  gl_Position = vec4(pos, 0.0, 1.0);

  Out.uv = pos.xy * 0.5 + 0.5;
  Out.uv.x *= 4.236; // Display 88 Keys
  Out.uv.x += 0.04;  // Move Keys To start from A
}