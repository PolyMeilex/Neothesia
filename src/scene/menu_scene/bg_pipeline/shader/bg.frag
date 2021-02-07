#version 450


layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;


layout(set=0, binding=0) 
uniform Uniforms2 {
    float u_time;
};

mat2 rotZ(float angle) {
  float ca = cos(angle);
  float sa = sin(angle);
  return mat2(ca, -sa, sa, ca);
}

vec3 note_render(vec2 uv, float pos, vec3 color) {
  float mod_x = mod(uv.x, 0.1 * 2.5 * 2.0);

  vec3 col = vec3(0.35,0.08,0.85);
  // vec3 col = vec3(160.0 / 255.0, 81.0 / 255.0, 238.0 / 255.0);

  if (pos == 0.5) {
    col = vec3(0.16,0.02,0.44);
    // col = vec3(113.0 / 255.0, 48.0 / 255.0, 178.0 / 255.0);
  }

  if (uv.y > 0.0 && uv.y < 0.5) {
    color = mix(color, col,
                smoothstep(-0.002, 0., 127. / 5800. - abs(mod_x - pos)));
  }
  
  return color;
}

#define speed -0.5
#define liveTime 2.6

void main() {
  vec2 st = uv;
  vec3 color = vec3(0.01);

  float d = 0.0;

  st *= rotZ(0.7);
  st.x *= 1.5;
  st.x = mod(st.x, 0.5);

  {
    st.y += 0.5;

    float off = 0.0;
    vec2 pos = st;

    pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
    color = note_render(pos, 0.1, color);

    off = 1.0;
    pos = st;
    pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
    color = note_render(pos, 0.1 * 2.0, color);

    off = 3.0;
    pos = st;
    pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
    color = note_render(pos, 0.1 * 3.0, color);

    off = 2.0;
    pos = st;
    pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
    color = note_render(pos, 0.1 * 4.0, color);

    off = 0.0;
    pos = st;
    pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
    color = note_render(pos, 0.1 * 5.0, color);

    off = 4.0;
    pos = st;
    pos.y -= mod((u_time * speed + off) / 5.0, 1.0) * liveTime;
    color = note_render(pos, 0.1 * 5.0, color);
  }

  f_color = vec4(color / 2.5, 0.5);
}