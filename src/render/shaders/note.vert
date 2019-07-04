#version 330

in vec2 pos;
in vec3 noteIn; // id,start,dur

#define notesCount 52.0

// uniform float n;
// uniform float startTime;
// uniform float duration;
uniform float time;

out INTERFACE {
  vec2 uv;
  vec2 size;
  float isBlack;
}
Out;

#define notesCount 52.0
#define speed 5.0

void main() {
  float n = noteIn.x - 24;
  int note = int(mod(n, 12));
  Out.isBlack =
      float(note == 1 || note == 3 || note == 6 || note == 8 || note == 10);

  Out.size = vec2(1.0 * 2.0 / notesCount, speed * noteIn.z);

  if (Out.isBlack == 1.0) {
    Out.size.x /= 2.0;
  }

  Out.uv = Out.size * pos;

  const float a = (1.0 / (notesCount - 1.0)) * (2.0 - 2.0 / notesCount);
  const float b = -1.0 + 1.0 / notesCount;
  // vec2 left = vec2(m * a + b, 0.0);

  vec2 offset = vec2(float(n + 2.0) * a + b + (Out.isBlack / notesCount / 1.5),
                     Out.size.y * 0.5 - 0.5 + speed * (noteIn.y - time));

  float whiteNoteSize = (0.9 * 2.0 / notesCount) + 0.003;
  float oct = floor(n / 12.0);
  offset.x -= (whiteNoteSize * 5.1) * oct;

  if (note == 0) {

  } else if (note <= 2) {
    offset.x -= whiteNoteSize * 1.0;
  } else if (note <= 5) {
    offset.x -= whiteNoteSize * 2.0;
  } else if (note <= 7) {
    offset.x -= whiteNoteSize * 3.0;
  } else if (note <= 9) {
    offset.x -= whiteNoteSize * 4.0;
  } else if (note <= 11) {
    offset.x -= whiteNoteSize * 5.0;
  }

  gl_Position = vec4(pos * Out.size + offset, 0.0, 1.0);

  // gl_Position = vec4(pos, 0.0, 1.0);
}