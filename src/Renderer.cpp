// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include "Renderer.h"
#include "Tga.h"
#include "OSGraphics.h"

#include <limits>

using namespace std;

// These are static because OpenGL is (essentially) static
static unsigned int last_texture_id = numeric_limits<unsigned int>::max();

void SelectTexture(unsigned int texture_id) {

  if (texture_id == last_texture_id)
    return;

  glBindTexture(GL_TEXTURE_2D, texture_id);
  last_texture_id = texture_id;
}


Renderer::Renderer(GLContext glcontext, PGContext pangocontext) :
  m_xoffset(0),
  m_yoffset(0),
  m_glcontext(glcontext),
  m_pangocontext(pangocontext) {
}

Color Renderer::ToColor(int r, int g, int b, int a) {

  Color c;
  c.r = r;
  c.g = g;
  c.b = b;
  c.a = a;

  return c;
}

void Renderer::SetVSyncInterval(int interval) {
// #ifdef WIN32

//    const char *extensions = reinterpret_cast<const char*>(static_cast<const unsigned char*>(glGetString( GL_EXTENSIONS )));

//    // Check if the WGL_EXT_swap_control extension is supported.
//    if (strstr(extensions, "WGL_EXT_swap_control") == 0) return;

//    typedef BOOL (APIENTRY *SWAP_INTERVAL_PROC)( int );
//    SWAP_INTERVAL_PROC wglSwapIntervalEXT = (SWAP_INTERVAL_PROC)wglGetProcAddress( "wglSwapIntervalEXT" );
//    if (wglSwapIntervalEXT) wglSwapIntervalEXT(interval);

// #else

//    GLint i = interval;
//    GLboolean ret = aglSetInteger(m_glcontext, AGL_SWAP_INTERVAL, &i);
//    if (ret == GL_FALSE)
//    {
//       // LOGTODO!
//       // This is non-critical.  V-Sync might just not be supported.
//    }

// #endif
}

void Renderer::SwapBuffers() {
  m_glcontext->get_gl_drawable()->swap_buffers();
}

void Renderer::ForceTexture(unsigned int texture_id) {
  last_texture_id = numeric_limits<unsigned int>::max();
  SelectTexture(texture_id);
}

void Renderer::SetColor(Color c) {
  SetColor(c.r, c.g, c.b, c.a);
}

void Renderer::SetColor(int r, int g, int b, int a) {
  glColor4f(r / 255.0f, g / 255.0f, b / 255.0f, a / 255.0f);
}

void Renderer::DrawQuad(int x, int y, int w, int h) {
  SelectTexture(0);

  glBegin(GL_QUADS);
  glVertex3i(   x + m_xoffset,   y + m_yoffset, 0);
  glVertex3i( x+w + m_xoffset,   y + m_yoffset, 0);
  glVertex3i( x+w + m_xoffset, y+h + m_yoffset, 0);
  glVertex3i(   x + m_xoffset, y+h + m_yoffset, 0);
  glEnd();
}

void Renderer::DrawTga(const Tga *tga, int x, int y) const {
  DrawTga(tga, x, y, (int)tga->GetWidth(), (int)tga->GetHeight(), 0, 0);
}

void Renderer::DrawTga(const Tga *tga, int in_x, int in_y, int width, int height,
                       int src_x, int src_y) const {

  const int x = in_x + m_xoffset;
  const int y = in_y + m_yoffset;

  const double tx = static_cast<double>(src_x) / static_cast<double>(tga->GetWidth());
  const double ty = -static_cast<double>(src_y) / static_cast<double>(tga->GetHeight());
  const double tw = static_cast<double>(width) / static_cast<double>(tga->GetWidth());
  const double th = -static_cast<double>(height)/ static_cast<double>(tga->GetHeight());

  SelectTexture(tga->GetId());
  glBegin(GL_QUADS);
  glTexCoord2d(   tx,    ty); glVertex3i(      x,        y, 0);
  glTexCoord2d(   tx, ty+th); glVertex3i(      x, y+height, 0);
  glTexCoord2d(tx+tw, ty+th); glVertex3i(x+width, y+height, 0);
  glTexCoord2d(tx+tw,    ty); glVertex3i(x+width,        y, 0);
  glEnd();
}

void Renderer::DrawStretchedTga(const Tga *tga, int x, int y, int w, int h) const {
  DrawStretchedTga(tga, x, y, w, h, 0, 0, (int)tga->GetWidth(), (int)tga->GetHeight());
}

void Renderer::DrawStretchedTga(const Tga *tga, int x, int y, int w, int h,
                                int src_x, int src_y, int src_w, int src_h) const {
  const int sx = x + m_xoffset;
  const int sy = y + m_yoffset;

  const double tx =  static_cast<double>(src_x) / static_cast<double>(tga->GetWidth());
  const double ty = -static_cast<double>(src_y) / static_cast<double>(tga->GetHeight());
  const double tw =  static_cast<double>(src_w) / static_cast<double>(tga->GetWidth());
  const double th = -static_cast<double>(src_h) / static_cast<double>(tga->GetHeight());

  SelectTexture(tga->GetId());

  glBegin(GL_QUADS);
  glTexCoord2d(   tx,    ty); glVertex3i(  sx,   sy, 0);
  glTexCoord2d(   tx, ty+th); glVertex3i(  sx, sy+h, 0);
  glTexCoord2d(tx+tw, ty+th); glVertex3i(sx+w, sy+h, 0);
  glTexCoord2d(tx+tw,    ty); glVertex3i(sx+w,   sy, 0);
  glEnd();
}
