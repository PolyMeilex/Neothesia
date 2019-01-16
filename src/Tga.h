// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#ifndef __TGA_H
#define __TGA_H

#include <string>

typedef unsigned int TextureId;

class Tga {
public:

  static Tga *Load(const std::string &resource_name);
  static void Release(Tga *tga);

  TextureId GetId() const {
    return m_texture_id;
  }

  unsigned int GetWidth() const {
    return m_width;
  }

  unsigned int GetHeight() const {
    return m_height;
  }

  void SetSmooth(bool smooth);

private:

  TextureId m_texture_id;
  unsigned int m_width;
  unsigned int m_height;

  Tga() { }
  ~Tga() { }

  Tga(const Tga& rhs);
  Tga &operator=(const Tga& rhs);


  static Tga *LoadFromData(const unsigned char *bytes);

  static Tga *LoadCompressed(const unsigned char *src, unsigned char *dest,
                             unsigned int width, unsigned int height, unsigned int bpp);

  static Tga *LoadUncompressed(const unsigned char *src, unsigned char *dest,
                               unsigned int size, unsigned int width, unsigned int height,
                               unsigned int bpp);

  static Tga *BuildFromParameters(const unsigned char *data, unsigned int width,
                                  unsigned int height, unsigned int bpp);
};

#endif // __TGA_H
