#version 330

out vec4 fragColor;

uniform ActiveNotes { uint activeNotesArray[128]; };

in INTERFACE { vec2 uv; }
In;

#define border 0.002

vec3 key_color;

void black_key_render(bool drawBase, vec2 uv, float pos, float pitch_offset,
                      float curr_pitch, inout vec3 color) {
  float mod_x = mod(uv.x + 4. * 59. / 725., 413. / 725.);
  float div_x = floor((uv.x + 4. * 59. / 725.) / (413. / 725.));

  vec3 col = vec3(0.0);
  bool isActive = false;

  if (!drawBase) {
    col = key_color / 1.5;
    isActive = !(curr_pitch != pitch_offset + div_x * 12. - 12.);
  }

  if (isActive || drawBase) {
    color = mix(
        color, col,
        smoothstep(-border, 0., 5. / 29. - abs(uv.y - .25 + 5. / 29.)) *
            smoothstep(-border, 0., 127. / 5800. - abs(mod_x - pos / 5800.)));
  }
}

void main() {
  vec3 color = vec3(1.0);
  // key_color = vec3(131.0 / 255.0, 23.0 / 255.0, 181.0 / 255.0);
  key_color = vec3(0.8);

  if (In.uv.y < .24) {
    float key_loc = floor(725. / 59. * (In.uv.x + 531. / 1450.)) - 7.0;

    float pitch =
        float((2 * int(key_loc) - int(floor((float(key_loc) + .5) / 3.5))));

    uint notes[128] = activeNotesArray;

    color = mix(
        vec3(0.0), color,
        smoothstep(0., border, abs(mod(In.uv.x, 59. / 725.) - 59. / 1450.)));

    for (int i = 0; i < 88; ++i) {
      float mouse_pitch = float(notes[i]) - 3.0;

      if (mouse_pitch == pitch) {
        color = key_color;
      }
    }

    // Draw Black Keys
    if (In.uv.y > .08) {
      black_key_render(true, In.uv, 183., 1., 0.0, color);
      black_key_render(true, In.uv, 743., 3., 0.0, color);
      black_key_render(true, In.uv, 1577., 6., 0.0, color);
      black_key_render(true, In.uv, 2115., 8., 0.0, color);
      black_key_render(true, In.uv, 2653., 10., 0.0, color);
    }

    // Draw Highlights
    for (int i = 0; i < 88; ++i) {
      float mouse_pitch = float(notes[i]) - 3.0;

      if (In.uv.y > .08) {
        black_key_render(false, In.uv, 183., 1., mouse_pitch, color);
        black_key_render(false, In.uv, 743., 3., mouse_pitch, color);
        black_key_render(false, In.uv, 1577., 6., mouse_pitch, color);
        black_key_render(false, In.uv, 2115., 8., mouse_pitch, color);
        black_key_render(false, In.uv, 2653., 10., mouse_pitch, color);
      }
    }

  } else {
    // color = vec3(0.0);
    discard;
  }

  fragColor = vec4(color, 1.0);
}