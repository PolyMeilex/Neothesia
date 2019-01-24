// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "TextWriter.h"
#include "Renderer.h"
#include "LinthesiaError.h"
#include "OSGraphics.h"
#include "UserSettings.h"

#include <map>
#include <X11/Xlib.h>

using namespace std;

// TODO: This should be deleted at shutdown
static map<int, int> font_size_lookup;

// TODO: This should be deleted at shutdown
static map<int, Pango::FontDescription*> font_lookup;

// Returns the most suitable font available on the platform
// or an empty string if no font is available;
static const std::string get_default_font();

TextWriter::TextWriter(int in_x, int in_y, Renderer &in_renderer,
                       bool in_centered, int in_size, string fontname) :
  x(in_x),
  y(in_y),
  size(in_size),
  original_x(0),
  last_line_height(0),
  centered(in_centered),
  renderer(in_renderer) {

  x += renderer.m_xoffset;
  original_x = x;

  y += renderer.m_yoffset;
  point_size = size;


  if (font_size_lookup[size] == 0) {

    // Get font from user settings
    // if (fontname.empty()) {
    //   string key = "font_desc";
    //   fontname = UserSetting::Get(key, "");
      
    //   // Or set it if there is no default
    //   if (fontname.empty()) {
    //     fontname = get_default_font();
    //     UserSetting::Set(key, fontname);
    //   }
    // }

    int list_start = glGenLists(128);
    fontname = STRING(fontname << " " << in_size);

    //@TODO Beter Font System!!!
    Pango::FontDescription* font_desc = new Pango::FontDescription("Roboto Medium");

   Glib::RefPtr<Pango::Font> ret = Gdk::GL::Font::use_pango_font(*font_desc, 0, 128, list_start);
   if (!ret)
     throw LinthesiaError("An error ocurred while trying to use use_pango_font() with "
                          "font '" + fontname + "'");

    font_size_lookup[size] = list_start;
    font_lookup[size] = font_desc;
  }
}

int TextWriter::get_point_size() {
  return point_size;
}

TextWriter& TextWriter::next_line() {
  y += max(last_line_height, get_point_size());
  x = original_x;

  last_line_height = 0;
  return *this;
}

TextWriter& Text::operator<<(TextWriter& tw) const {
  int draw_x;
  int draw_y;
  calculate_position_and_advance_cursor(tw, &draw_x, &draw_y);

  string narrow(m_text.begin(), m_text.end());

  glBindTexture(GL_TEXTURE_2D, 0);

  glPushMatrix();
  tw.renderer.SetColor(m_color);
  glListBase(font_size_lookup[tw.size]);
  glRasterPos2i(draw_x, draw_y + tw.size);
  glCallLists(static_cast<int>(narrow.length()), GL_UNSIGNED_BYTE, narrow.c_str());
  glPopMatrix();

  // TODO: Should probably delete these on shutdown.
  //glDeleteLists(1000, 128);

  return tw;
}

void Text::calculate_position_and_advance_cursor(TextWriter &tw, int *out_x, int *out_y) const  {

  Glib::RefPtr<Pango::Layout> layout = Pango::Layout::create(tw.renderer.m_pangocontext);
  layout->set_text(m_text);
  layout->set_font_description(*(font_lookup[tw.size]));

  Pango::Rectangle drawing_rect = layout->get_pixel_logical_extents();
  tw.last_line_height = drawing_rect.get_height();

  if (tw.centered)
    *out_x = tw.x - drawing_rect.get_width() / 2;

  else {
    *out_x = tw.x;
    tw.x += drawing_rect.get_width();
  }

  *out_y = tw.y;
}

TextWriter& operator<<(TextWriter& tw, const Text& t) {
  return t.operator <<(tw);
}

TextWriter& newline(TextWriter& tw) {
  return tw.next_line();
}

TextWriter& operator<<(TextWriter& tw, const string& s)        { return tw << Text(s, White); }
TextWriter& operator<<(TextWriter& tw, const int& i)           { return tw << Text(i, White); }
TextWriter& operator<<(TextWriter& tw, const unsigned int& i)  { return tw << Text(i, White); }
TextWriter& operator<<(TextWriter& tw, const long& l)          { return tw << Text(l, White); }
TextWriter& operator<<(TextWriter& tw, const unsigned long& l) { return tw << Text(l, White); }

static
const std::string get_default_font()
{
  // populate a vector of candidates with the most common choices
  vector< string > allCandidates;
  allCandidates.push_back("serif");
  allCandidates.push_back("sans");
  allCandidates.push_back("clean");
  allCandidates.push_back("courier"); // Debian suggests using courier

  vector< string >::const_iterator candidate;
  const vector< string >::const_iterator end = allCandidates.end();

  // retrieve all fonts from the X server
  Display * const display = XOpenDisplay(NULL);
  int nbFonts = 0, i = 0;
  char ** const allFonts = XListFonts(display, "-*", 32767, &nbFonts);

  string returnedFont = (nbFonts > 0) ? allFonts[0] : "";

  // check if we have a candidate, and returns it if we do
  string currentFont;
  bool found = false;
  for (i = 0; i < nbFonts && !found; ++i)
  {
    currentFont = allFonts[i];

    for (candidate = allCandidates.begin();
         candidate != end && !found; ++candidate)
    {
      // any font that contains the name of the candidate ( "serif" ) will do
      if (currentFont.find(*candidate) != string::npos)
      {
        returnedFont = *candidate;
        found = true;
      }
    }
  }
  // raise(SIGINT);
  XFreeFontNames(allFonts);
  XCloseDisplay(display);

  return returnedFont;
}
