// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "KeyboardDisplay.h"
#include "LinthesiaError.h"
#include "StringUtil.h"
#include "TrackProperties.h"

#include "Renderer.h"
#include "Textures.h"
#include "Tga.h"

#include "GLShader.h"

#include "neolib/ParticleSystem.h"
// #include "neolib/FBO.h"

using namespace std;

const KeyboardDisplay::NoteTexDimensions KeyboardDisplay::WhiteNoteDimensions =
    {32, 128, 4, 25, 22, 28, 93, 100};
const KeyboardDisplay::NoteTexDimensions KeyboardDisplay::BlackNoteDimensions =
    {32, 64, 8, 20, 3, 8, 49, 55};

struct KeyTexDimensions {
  int tex_width;
  int tex_height;

  int left;
  int right;

  int top;
  int bottom;
};

GLuint blurProgram;
GLuint normalProgram;

// GLuint FramebufferName = 0;
// GLuint renderedTexture;

ParticleSystem particleSystem;

std::shared_ptr<NeoFBO> particlesFBO;
std::shared_ptr<NeoFBO> blurParticlesFBO;

const KeyboardDisplay::KeyTexDimensions KeyboardDisplay::BlackKeyDimensions = {
    32, 128, 8, 20, 15, 109};

KeyboardDisplay::KeyboardDisplay(KeyboardSize size, int pixelWidth,
                                 int pixelHeight, int stateX_, int stateY_)
    : m_size(size), m_width(pixelWidth), m_height(pixelHeight), stateX(stateX_),
      stateY(stateY_) {


  particlesFBO = std::shared_ptr<NeoFBO> (new NeoFBO(stateX,stateY));
  blurParticlesFBO = std::shared_ptr<NeoFBO> (new NeoFBO(stateX,stateY));

  GLShadersInit();

  normalProgram = LoadShader(RenderTextureGLShader);
  blurProgram = LoadShader(BlurParticleGLShader);

}

bool nofirst;

GLuint quad_VertexArrayID;
GLuint quad_vertexbuffer;

std::vector<float> quadVertices{-1.0, -1.0, 0.0, 1.0, -1.0, 0.0,
                                -1.0, 1.0,  0.0, 1.0, 1.0,  0.0};

std::vector<unsigned int> quadIndices{0, 1, 2, 2, 1, 3};

GLuint _vao;
GLuint _ebo;

size_t _count;

void sQuad(int stateX, int stateY, GLuint textToBlur,GLuint program) {

  if (!nofirst) {
    _count = quadIndices.size();
    GLuint vbo = 0;
    glGenBuffers(1, &vbo);
    glBindBuffer(GL_ARRAY_BUFFER, vbo);

    glBufferData(GL_ARRAY_BUFFER, sizeof(GLfloat) * quadVertices.size(),
                 &(quadVertices[0]), GL_STATIC_DRAW);

    _vao = 0;
    glGenVertexArrays(1, &_vao);
    glBindVertexArray(_vao);
      // The first attribute will be the vertices positions.
      glEnableVertexAttribArray(0);
      glBindBuffer(GL_ARRAY_BUFFER, vbo);
      glVertexAttribPointer(0, 3, GL_FLOAT, GL_FALSE, 0, NULL);

      // We load the indices data
      glGenBuffers(1, &_ebo);
      glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, _ebo);
      glBufferData(GL_ELEMENT_ARRAY_BUFFER,
                  sizeof(unsigned int) * quadIndices.size(), &(quadIndices[0]),
                  GL_STATIC_DRAW);

    glBindVertexArray(0);


    // glBindTexture(GL_TEXTURE_2D, renderedTexture);
    // GLuint texUniID = glGetUniformLocation(program, "screenTexture");
    // glUniform1i(texUniID, 0);

    nofirst = true;
  }

  glUseProgram(program);
  
    glActiveTexture(GL_TEXTURE0);
    glBindTexture(GL_TEXTURE_2D, textToBlur);

    glBindVertexArray(_vao);

    glBindBuffer(GL_ELEMENT_ARRAY_BUFFER, _ebo);
    glDrawElements(GL_TRIANGLES, _count, GL_UNSIGNED_INT, (void *)0);

    glBindVertexArray(0);

  glUseProgram(0);
}

void KeyboardDisplay::Draw(Renderer &renderer, const Tga *key_tex[4],
                           const Tga *note_tex[4], int x, int y,
                           const TranslatedNoteSet &notes,
                           microseconds_t show_duration,
                           microseconds_t current_time,
                           const vector<Track::Properties> &track_properties,
                           const MidiEventMicrosecondList &bar_line_usecs) {
  glViewport(0, 0, stateX, stateY);

  // Source: Measured from Yamaha P-70
  const static double WhiteWidthHeightRatio = 6.8181818;
  const static double BlackWidthHeightRatio = 7.9166666;
  const static double WhiteBlackWidthRatio = 0.5454545;

  const int white_key_count = GetWhiteKeyCount();

  // Calculate the largest white key size we can, and then
  // leave room for a single pixel space between each key
  int white_width = (m_width / white_key_count) - 1;
  int white_space = 1;

  int white_height = static_cast<int>(white_width * WhiteWidthHeightRatio);

  const int black_width = static_cast<int>(white_width * WhiteBlackWidthRatio);
  const int black_height =
      static_cast<int>(black_width * BlackWidthHeightRatio);
  const int black_offset = white_width - (black_width / 2);

  // The dimensions given to the keyboard object are bounds.  Because of pixel
  // rounding, the keyboard will usually occupy less than the maximum in
  // either direction.
  //
  // So, we just try to center the keyboard inside the bounds.
  const int final_width = (white_width + white_space) * white_key_count;
  const int x_offset = (m_width - final_width) / 2;
  const int y_offset = (m_height - white_height);

  // Give the notes a little more room to work with so they can roll under
  // the keys without distortion
  const int y_roll_under = white_height * 3 / 4;

  // Symbolic names for the arbitrary array passed in here
  enum { Rail, Shadow, BlackKey };

  DrawGuides(renderer, white_key_count, white_width, white_space, x + x_offset,y, y_offset);

  particleSystem.UpdateParticles();

  particlesFBO -> Bind();
    glViewport(0, 0, stateX, stateY);

    sQuad(stateX, stateY,blurParticlesFBO -> GetTexture(),normalProgram);

    particleSystem.DrawParticles(renderer);


    renderer.SetColor(Renderer::ToColor(255, 255, 255));

  particlesFBO -> Unbind();

  blurParticlesFBO -> Bind();
    glViewport(0, 0, stateX, stateY);

    sQuad(stateX, stateY,particlesFBO -> GetTexture(),blurProgram);

  blurParticlesFBO -> Unbind();
  
  
  
  sQuad(stateX, stateY,particlesFBO -> GetTexture(),normalProgram);

  

  renderer.SetColor(Renderer::ToColor(255, 255, 255));


  DrawNotePass(renderer, note_tex[2], note_tex[3], white_width, white_space,
               black_width, black_offset, x + x_offset, y, y_offset,
               y_roll_under, notes, show_duration, current_time,
               track_properties);

  const int ActualKeyboardWidth =
      white_width * white_key_count + white_space * (white_key_count - 1);

  // Black out the background of where the keys are about to appear

  renderer.SetColor(Renderer::ToColor(0, 0, 0));
  renderer.DrawQuad(x + x_offset, y + y_offset, ActualKeyboardWidth,
                    white_height);

 

  DrawShadow(renderer, key_tex[Shadow], x + x_offset,
             y + y_offset + white_height - 10, ActualKeyboardWidth);
  DrawWhiteKeys(renderer, key_tex[3], false, white_key_count, white_width,
                white_height, white_space, x + x_offset, y + y_offset);
  DrawBlackKeys(renderer, key_tex[BlackKey], false, white_key_count,
                white_width, black_width, black_height, white_space,
                x + x_offset, y + y_offset, black_offset);
  DrawShadow(renderer, key_tex[Shadow], x + x_offset, y + y_offset,ActualKeyboardWidth);
  DrawRail(renderer, key_tex[Rail], x + x_offset, y + y_offset,ActualKeyboardWidth);


  

}

int KeyboardDisplay::GetStartingOctave() const {

  // Source: Various "Specification" pages at Yamaha's website
  const static int StartingOctaveOn37 = 2;
  const static int StartingOctaveOn49 = 1;
  const static int StartingOctaveOn61 = 1; // TODO!
  const static int StartingOctaveOn76 = 0; // TODO!
  const static int StartingOctaveOn88 = 0;

  switch (m_size) {
  case KeyboardSize37:
    return StartingOctaveOn37;
  case KeyboardSize49:
    return StartingOctaveOn49;
  case KeyboardSize61:
    return StartingOctaveOn61;
  case KeyboardSize76:
    return StartingOctaveOn76;
  case KeyboardSize88:
    return StartingOctaveOn88;
  default:
    throw LinthesiaError(Error_BadPianoType);
  }
}

char KeyboardDisplay::GetStartingNote() const {

  // Source: Various "Specification" pages at Yamaha's website
  const static char StartingKeyOn37 = 'F'; // F3-F6
  const static char StartingKeyOn49 = 'C'; // C3-C6
  const static char StartingKeyOn61 = 'C'; // C1-C6 // TODO!
  const static char StartingKeyOn76 = 'E'; // E0-G6 // TODO!
  const static char StartingKeyOn88 = 'A'; // A0-C6

  switch (m_size) {
  case KeyboardSize37:
    return StartingKeyOn37;
  case KeyboardSize49:
    return StartingKeyOn49;
  case KeyboardSize61:
    return StartingKeyOn61;
  case KeyboardSize76:
    return StartingKeyOn76;
  case KeyboardSize88:
    return StartingKeyOn88;
  default:
    throw LinthesiaError(Error_BadPianoType);
  }
}

int KeyboardDisplay::GetWhiteKeyCount() const {

  // Source: Google Image Search
  const static int WhiteKeysOn37 = 22;
  const static int WhiteKeysOn49 = 29;
  const static int WhiteKeysOn61 = 36;
  const static int WhiteKeysOn76 = 45;
  const static int WhiteKeysOn88 = 52;

  switch (m_size) {
  case KeyboardSize37:
    return WhiteKeysOn37;
  case KeyboardSize49:
    return WhiteKeysOn49;
  case KeyboardSize61:
    return WhiteKeysOn61;
  case KeyboardSize76:
    return WhiteKeysOn76;
  case KeyboardSize88:
    return WhiteKeysOn88;
  default:
    throw LinthesiaError(Error_BadPianoType);
  }
}

void KeyboardDisplay::DrawWhiteKeys(Renderer &renderer, const Tga *tex,
                                    bool active_only, int key_count,
                                    int key_width, int key_height,
                                    int key_space, int x_offset,
                                    int y_offset) const {
  Color white = Renderer::ToColor(255, 255, 255);

  char current_white = GetStartingNote();
  int current_octave = GetStartingOctave() + 1;
  for (int i = 0; i < key_count; ++i) {

    // Check to see if this is one of the active notes
    const string note_name = STRING(current_white << current_octave);

    KeyNames::const_iterator find_result = m_active_keys.find(note_name);
    bool active = (find_result != m_active_keys.end());

    Color c = white;

    if (active)
      c = Track::ColorNoteWhite[find_result->second];

    if ((active_only && active) || !active_only) {
      renderer.SetColor(c);

      const int key_x = i * (key_width + key_space) + x_offset;

      int press = 0;

      if (active) {
        particleSystem.SpawnParticle(key_x + key_width / 2, y_offset, c);
        press = 3;
      }

      renderer.DrawStretchedTga(tex, key_x, y_offset, key_width,
                                key_height + press, 0, 0, 32, 128);
    }

    current_white++;

    if (current_white == 'H')
      current_white = 'A';

    if (current_white == 'C')
      current_octave++;
  }

  renderer.SetColor(white);
}

void KeyboardDisplay::DrawBlackKey(Renderer &renderer, const Tga *tex,
                                   const KeyTexDimensions &tex_dimensions,
                                   int x, int y, int w, int h,
                                   Track::TrackColor color) const {

  const KeyTexDimensions &d = tex_dimensions;

  const int tex_w = d.right - d.left;
  const double width_scale = double(w) / double(tex_w);
  const double full_tex_width = d.tex_width * width_scale;
  const double left_offset = d.left * width_scale;

  const int src_x = (int(color) * d.tex_width);
  const int dest_x = int(x - left_offset) - 1;
  const int dest_w = int(full_tex_width);

  const int tex_h = d.bottom - d.top;
  const double height_scale = double(h) / double(tex_h);
  const double full_tex_height = d.tex_height * height_scale;
  const double top_offset = d.top * height_scale;

  const int dest_y = int(y - top_offset) - 1;
  const int dest_h = int(full_tex_height);

  renderer.DrawStretchedTga(tex, dest_x, dest_y, dest_w, dest_h, src_x, 0,
                            d.tex_width, d.tex_height);
}

void KeyboardDisplay::DrawBlackKeys(Renderer &renderer, const Tga *tex,
                                    bool active_only, int white_key_count,
                                    int white_width, int black_width,
                                    int black_height, int key_space,
                                    int x_offset, int y_offset,
                                    int black_offset) const {

  char current_white = GetStartingNote();
  int current_octave = GetStartingOctave() + 1;
  for (int i = 0; i < white_key_count; ++i) {
    // Don't allow a very last black key
    if (i == white_key_count - 1)
      break;

    switch (current_white) {
    case 'A':
    case 'C':
    case 'D':
    case 'F':
    case 'G': {
      // Check to see if this is one of the active notes
      const string note_name = STRING(current_white << '#' << current_octave);

      KeyNames::const_iterator find_result = m_active_keys.find(note_name);
      bool active = (find_result != m_active_keys.end());

      // In this case, MissedNote isn't actually MissedNote.  In the black key
      // texture we use this value (which doesn't make any sense in this
      // context) as the default "Black" color.
      Track::TrackColor c = Track::MissedNote;
      if (active)
        c = find_result->second;

      if (!active_only || (active_only && active)) {
        const int start_x =
            i * (white_width + key_space) + x_offset + black_offset;

        if (active)
          particleSystem.SpawnParticle(start_x + black_width / 2, y_offset,
                                       Track::ColorNoteWhite[c]);
        DrawBlackKey(renderer, tex, BlackKeyDimensions, start_x, y_offset,
                     black_width, black_height, c);
      }
    }
    }

    current_white++;
    if (current_white == 'H')
      current_white = 'A';

    if (current_white == 'C')
      current_octave++;
  }
}

void DrawWidthStretched(Renderer &renderer, const Tga *tex, int x, int y,
                        int width) {

  renderer.DrawStretchedTga(tex, x, y, width, tex->GetHeight(), 0, 0,
                            tex->GetWidth(), tex->GetWidth());
}

void KeyboardDisplay::DrawRail(Renderer &renderer, const Tga *tex, int x, int y,
                               int width) const {

  const static int RailOffsetY = -4;
  DrawWidthStretched(renderer, tex, x, y + RailOffsetY, width);
}

void KeyboardDisplay::DrawShadow(Renderer &renderer, const Tga *tex, int x,
                                 int y, int width) const {

  const static int ShadowOffsetY = 10;
  DrawWidthStretched(renderer, tex, x, y + ShadowOffsetY, width);
}

void KeyboardDisplay::DrawGuides(Renderer &renderer, int key_count,
                                 int key_width, int key_space, int x_offset,
                                 int y, int y_offset) const {

  const static int PixelsOffKeyboard = 2;
  int keyboard_width = key_width * key_count + key_space * (key_count - 1);

  const Color thick(Renderer::ToColor(0x47, 0x47, 0x47));
  const Color thin(Renderer::ToColor(0x47, 0x47, 0x47));

  char current_white = GetStartingNote() - 1;
  int current_octave = GetStartingOctave() + 1;
  for (int i = 0; i < key_count + 1; ++i) {
    const int key_x = i * (key_width + key_space) + x_offset - 1;

    int guide_thickness = 1;
    Color guide_color = thin;

    bool draw_guide = true;
    switch (current_white) {
    case 'B':
      guide_color = thick;
      guide_thickness = 2;
      break;
    case 'E':
      guide_color = thin;
      break;
    default:
      draw_guide = false;
      break;
    }

    if (draw_guide) {
      renderer.SetColor(guide_color);
      renderer.DrawQuad(key_x - guide_thickness / 2, y, guide_thickness,
                        y_offset - PixelsOffKeyboard);
    }

    current_white++;
    if (current_white == 'H')
      current_white = 'A';
    if (current_white == 'C')
      current_octave++;
  }
}

void KeyboardDisplay::DrawBars(
    Renderer &renderer, int x, int y, int y_offset, int y_roll_under,
    int final_width, microseconds_t show_duration, microseconds_t current_time,
    const MidiEventMicrosecondList &bar_line_usecs) const {
  int i = 0;
  MidiEventMicrosecondList::const_iterator j = bar_line_usecs.begin();
  const Color bar_color(Renderer::ToColor(0x50, 0x50, 0x50));
  const Color text_color1(Renderer::ToColor(0x50, 0x50, 0x50));
  const Color text_color2(Renderer::ToColor(0x90, 0x90, 0x90));
  for (; j != bar_line_usecs.end(); ++j, ++i) {
    renderer.SetColor(bar_color);
    microseconds_t bar_usec = *j;
    // Skip previous bars
    if (bar_usec < current_time)
      continue;
    // This list is sorted by note start time.  The moment we encounter
    // a bar scrolled off the window, we're done drawing
    if (bar_usec > current_time + show_duration)
      break;

    const double scaling_factor =
        static_cast<double>(y_offset) / static_cast<double>(show_duration);

    const long long roll_under =
        static_cast<int>(y_roll_under / scaling_factor);
    const long long adjusted_offset = max(bar_usec - current_time, -roll_under);

    // Convert our times to pixel coordinates
    const int y_bar_offset =
        y - static_cast<int>(adjusted_offset * scaling_factor) + y_offset;
    renderer.DrawQuad(x, y_bar_offset, final_width, 2);

    // Add a label with a bar number
    // Background text
    TextWriter bar_writer2(x + 3, y_bar_offset - 13, renderer, false, 11);
    bar_writer2 << Text(STRING(i + 1), text_color1);

    TextWriter bar_writer3(x + 5, y_bar_offset - 15, renderer, false, 11);
    bar_writer3 << Text(STRING(i + 1), text_color1);

    TextWriter bar_writer4(x + 3, y_bar_offset - 15, renderer, false, 11);
    bar_writer4 << Text(STRING(i + 1), text_color1);

    TextWriter bar_writer5(x + 5, y_bar_offset - 13, renderer, false, 11);
    bar_writer5 << Text(STRING(i + 1), text_color1);

    // Foreground text
    TextWriter bar_writer(x + 4, y_bar_offset - 14, renderer, false, 11);
    bar_writer << Text(STRING(i + 1), text_color2);
  }
}

void KeyboardDisplay::DrawNote(Renderer &renderer, const Tga *tex,
                               const NoteTexDimensions &tex_dimensions, int x,
                               int y, int w, int h, int color_id) const {

  const NoteTexDimensions &d = tex_dimensions;

  // Width is super-easy
  const int tex_note_w = d.right - d.left;

  const double width_scale = double(w) / double(tex_note_w);
  const double full_tex_width = d.tex_width * width_scale;
  const double left_offset = d.left * width_scale;

  const int src_x = (color_id * d.tex_width);
  const int dest_x = int(x - left_offset);
  const int dest_w = int(full_tex_width);

  // Now we draw the note in three sections:
  // - Crown (fixed (relative) height)
  // - Middle (variable height)
  // - Heel (fixed (relative) height)

  // Force the note to be at least as large as the crown + heel height
  const double crown_h = (d.crown_end - d.crown_start) * width_scale;
  const double heel_h = (d.heel_end - d.heel_start) * width_scale;
  const double min_height = crown_h + heel_h + 1.0;

  if (h < min_height) {
    const int diff = int(min_height - h);
    h += diff;
    y -= diff;
  }

  // We actually use the width scale in height calculations
  // to keep the proportions fixed.
  const double crown_start_offset = d.crown_start * width_scale;
  const double crown_end_offset = d.crown_end * width_scale;
  const double heel_height = double(d.heel_end - d.heel_start) * width_scale;
  const double bottom_height = double(d.tex_height - d.heel_end) * width_scale;

  const int dest_y1 = int(y - crown_start_offset);
  const int dest_y2 = int(dest_y1 + crown_end_offset);
  const int dest_y3 = int((y + h) - heel_height);
  const int dest_y4 = int(dest_y3 + bottom_height);

  renderer.DrawStretchedTga(tex, dest_x, dest_y1, dest_w, dest_y2 - dest_y1,
                            src_x, 0, d.tex_width, d.crown_end);
  renderer.DrawStretchedTga(tex, dest_x, dest_y2, dest_w, dest_y3 - dest_y2,
                            src_x, d.crown_end, d.tex_width,
                            d.heel_start - d.crown_end);

  renderer.DrawStretchedTga(tex, dest_x, dest_y3, dest_w, dest_y4 - dest_y3,
                            src_x, d.heel_start, d.tex_width,
                            d.tex_height - d.heel_start);
}

void KeyboardDisplay::DrawNotePass(
    Renderer &renderer, const Tga *tex_white, const Tga *tex_black,
    int white_width, int key_space, int black_width, int black_offset,
    int x_offset, int y, int y_offset, int y_roll_under,
    const TranslatedNoteSet &notes, microseconds_t show_duration,
    microseconds_t current_time,
    const vector<Track::Properties> &track_properties) const {

  // Shiny music domain knowledge
  const static unsigned int NotesPerOctave = 12;
  const static unsigned int WhiteNotesPerOctave = 7;
  const static bool IsBlackNote[12] = {false, true,  false, true,
                                       false, false, true,  false,
                                       true,  false, true,  false};

  // The constants used in the switch below refer to the number
  // of white keys off 'C' that type of piano starts on
  int keyboard_type_offset = 0;

  switch (m_size) {
  case KeyboardSize37:
    keyboard_type_offset = 4 - WhiteNotesPerOctave;
    break;
  case KeyboardSize49:
    keyboard_type_offset = 0 - WhiteNotesPerOctave;
    break;
  case KeyboardSize61:
    keyboard_type_offset = 7 - WhiteNotesPerOctave;
    break; // TODO!
  case KeyboardSize76:
    keyboard_type_offset = 5 - WhiteNotesPerOctave;
    break; // TODO!
  case KeyboardSize88:
    keyboard_type_offset = 2 - WhiteNotesPerOctave;
    break;
  default:
    throw LinthesiaError(Error_BadPianoType);
  }

  // This array describes how to "stack" notes in a single place.  The
  // IsBlackNote array then tells which one should be shifted slightly to the
  // right
  const static int NoteToWhiteNoteOffset[12] = {0,  -1, -1, -2, -2, -2,
                                                -3, -3, -4, -4, -5, -5};

  const static int MinNoteHeight = 3;

  bool drawing_black = false;
  for (int toggle = 0; toggle < 2; ++toggle) {

    for (TranslatedNoteSet::const_iterator i = notes.begin(); i != notes.end();
         ++i) {
      // This list is sorted by note start time.  The moment we encounter
      // a note scrolled off the window, we're done drawing
      if (i->start > current_time + show_duration)
        break;

      const Track::Mode mode = track_properties[i->track_id].mode;
      if (mode == Track::ModeNotPlayed || mode == Track::ModePlayedButHidden)
        continue;

      const int octave = (i->note_id / NotesPerOctave) - GetStartingOctave();
      const int octave_base = i->note_id % NotesPerOctave;
      const int stack_offset = NoteToWhiteNoteOffset[octave_base];
      const bool is_black = IsBlackNote[octave_base];

      if (drawing_black != is_black)
        continue;

      const int octave_offset = (max(octave - 1, 0) * WhiteNotesPerOctave);
      const int inner_octave_offset = (octave_base + stack_offset);
      const int generalized_black_offset = (is_black ? black_offset : 0);

      const double scaling_factor =
          static_cast<double>(y_offset) / static_cast<double>(show_duration);

      const long long roll_under =
          static_cast<int>(y_roll_under / scaling_factor);
      const long long adjusted_start =
          max(i->start - current_time, -roll_under);
      const long long adjusted_end = max(i->end - current_time, 0LL);

      if (adjusted_end < adjusted_start)
        continue;

      // Convert our times to pixel coordinates
      const int y_end =
          y - static_cast<int>(adjusted_start * scaling_factor) + y_offset;
      const int y_start =
          y - static_cast<int>(adjusted_end * scaling_factor) + y_offset;

      const int start_x =
          (octave_offset + inner_octave_offset + keyboard_type_offset) *
              (white_width + key_space) +
          generalized_black_offset + x_offset;

      const int left = start_x - 1;
      const int top = y_start;
      const int width = (is_black ? black_width : white_width) + 2;
      int height = y_end - y_start;

      // Force a note to be a minimum height at all times
      // except when scrolling off underneath the keyboard and
      // coming in from the top of the screen.
      const bool hitting_bottom = (adjusted_start + current_time != i->start);
      const bool hitting_top = (adjusted_end + current_time != i->end);

      if (!hitting_bottom && !hitting_top) {
        while ((height) < MinNoteHeight)
          height++;
      }

      const Track::TrackColor color = track_properties[i->track_id].color;
      const int &brush_id =
          (((i->state == UserMissed) || (i->retry_state == UserMissed))
               ? Track::MissedNote
               : color);

      DrawNote(renderer, (drawing_black ? tex_black : tex_white),
               (drawing_black ? BlackNoteDimensions : WhiteNoteDimensions),
               left, top, width, height, brush_id);
    }

    drawing_black = !drawing_black;
  }
}

void KeyboardDisplay::SetKeyActive(const string &key_name, bool active,
                                   Track::TrackColor color) {
  if (active)
    m_active_keys[key_name] = color;

  else
    m_active_keys.erase(key_name);
}
