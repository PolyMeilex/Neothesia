// -*- mode: c++; coding: utf-8 -*-

// Linthesia

// Copyright (c) 2007 Nicholas Piegdon
// Adaptation to GNU/Linux by Oscar Aceña
// See COPYING for license information

#include <iostream>
#include <fstream>
#include <string.h> // Fix Artifact ID: 2927098

#include "Tga.h"
#include "OSGraphics.h"
#include "StringUtil.h"
#include "LinthesiaError.h"

#ifndef GRAPHDIR
#define GRAPHDIR "../graphics"
#endif

using namespace std;

Tga* Tga::Load(const string &resource_name) {

  // Append extension
  string full_name = resource_name + ".tga";

  // FIXME this!
  full_name = string(GRAPHDIR) + "/" + full_name;

  ifstream file(full_name.c_str(), ios::in|ios::binary|ios::ate);
  if (!file.is_open())
    throw LinthesiaError("Couldn't open TGA resource (" + full_name + ").");

  int size = file.tellg();
  unsigned char *bytes = new unsigned char[size];

  file.seekg(0, ios::beg);
  file.read((char*)bytes, size);
  file.close();

  Tga *ret = LoadFromData(bytes);
  delete[] bytes;

  ret->SetSmooth(false);
  return ret;
}

void Tga::Release(Tga *tga) {
  if (!tga)
    return;

  glDeleteTextures(1, &tga->m_texture_id);
  delete tga;
}

const static int TgaTypeHeaderLength = 12;
const static unsigned char UncompressedTgaHeader[TgaTypeHeaderLength] = {0,0,2,0,0,0,0,0,0,0,0,0};
const static unsigned char CompressedTgaHeader[TgaTypeHeaderLength] = {0,0,10,0,0,0,0,0,0,0,0,0};

void Tga::SetSmooth(bool smooth) {
  GLint filter = GL_NEAREST;
  if (smooth)
    filter = GL_LINEAR;

  glBindTexture(GL_TEXTURE_2D, m_texture_id);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, filter);
  glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, filter);
}

enum TgaType {

  TgaUncompressed,
  TgaCompressed,
  TgaUnknown
};

Tga *Tga::LoadFromData(const unsigned char *bytes) {
  if (!bytes)
    return 0;

  const unsigned char *pos = bytes;

  TgaType type = TgaUnknown;
  if (memcmp(UncompressedTgaHeader, pos, TgaTypeHeaderLength) == 0)
    type = TgaUncompressed;

  if (memcmp(CompressedTgaHeader, pos, TgaTypeHeaderLength) == 0)
    type = TgaCompressed;

  if (type == TgaUnknown)
    throw LinthesiaError("Unsupported TGA type.");

  // We're done with the type header
  pos += TgaTypeHeaderLength;

  unsigned int width = pos[1] * 256 + pos[0];
  unsigned int height = pos[3] * 256 + pos[2];
  unsigned int bpp = pos[4];

  // We're done with the data header
  const static int TgaDataHeaderLength = 6;
  pos += TgaDataHeaderLength;

  if (width <= 0 || height <= 0)
    throw LinthesiaError("Invalid TGA dimensions.");

  if (bpp != 24 && bpp != 32)
    throw LinthesiaError("Unsupported TGA BPP.");

  const unsigned int data_size = width * height * bpp/8;
  unsigned char *image_data = new unsigned char[data_size];

  Tga *t = 0;
  if (type == TgaCompressed)
    t = LoadCompressed(pos, image_data, width, height, bpp);

  if (type == TgaUncompressed)
    t = LoadUncompressed(pos, image_data, data_size, width, height, bpp);

  delete[] image_data;
  return t;
}

Tga *Tga::LoadUncompressed(const unsigned char *src, unsigned char *dest,
			   unsigned int size, unsigned int width, unsigned int height,
                           unsigned int bpp) {
  // We can use most of the data as-is with little modification
  memcpy(dest, src, size);

  for (unsigned int cswap = 0; cswap < size; cswap += bpp/8) {
    dest[cswap] ^= dest[cswap+2] ^= dest[cswap] ^= dest[cswap+2];
  }

  return BuildFromParameters(dest, width, height, bpp);
}

Tga *Tga::LoadCompressed(const unsigned char *src, unsigned char *dest,
			 unsigned int width, unsigned int height, unsigned int bpp) {

  const unsigned char *pos = src;

  const unsigned int BytesPerPixel = bpp / 8;
  const unsigned int PixelCount = height * width;

  const static unsigned int MaxBytesPerPixel = 4;
  unsigned char pixel_buffer[MaxBytesPerPixel];

  unsigned int pixel = 0;
  unsigned int byte = 0;

  while (pixel < PixelCount) {
    unsigned char chunkheader = 0;
    memcpy(&chunkheader, pos, sizeof(unsigned char));
    pos += sizeof(unsigned char);

    if (chunkheader < 128) {
      chunkheader++;

      for (short i = 0; i < chunkheader; i++) {
	memcpy(pixel_buffer, pos, BytesPerPixel);
	pos += BytesPerPixel;

	dest[byte + 0] = pixel_buffer[2];
	dest[byte + 1] = pixel_buffer[1];
	dest[byte + 2] = pixel_buffer[0];

	if (BytesPerPixel == 4)
	  dest[byte + 3] = pixel_buffer[3];

	byte += BytesPerPixel;
	pixel++;

	if (pixel > PixelCount)
	  throw LinthesiaError("Too many pixels in TGA.");
      }
    }

    else {
      chunkheader -= 127;

      memcpy(pixel_buffer, pos, BytesPerPixel);
      pos += BytesPerPixel;

      for (short i = 0; i < chunkheader; i++) {

	dest[byte + 0] = pixel_buffer[2];
	dest[byte + 1] = pixel_buffer[1];
	dest[byte + 2] = pixel_buffer[0];

	if (BytesPerPixel == 4)
	  dest[byte + 3] = pixel_buffer[3];

	byte += BytesPerPixel;
	pixel++;

	if (pixel > PixelCount)
	  throw LinthesiaError("Too many pixels in TGA.");

      }
    }
  }

  return BuildFromParameters(dest, width, height, bpp);
}


Tga *Tga::BuildFromParameters(const unsigned char *raw, unsigned int width,
			      unsigned int height, unsigned int bpp) {

  unsigned int pixel_format = 0;
  if (bpp == 24)
    pixel_format = GL_RGB;

  if (bpp == 32)
    pixel_format = GL_RGBA;

  TextureId id;
  glGenTextures(1, &id);
  if (!id)
    return 0;

  glBindTexture(GL_TEXTURE_2D, id);
  glPixelStorei(GL_UNPACK_ALIGNMENT, 4);
  glTexImage2D(GL_TEXTURE_2D, 0, bpp/8, width, height,
	       0, pixel_format, GL_UNSIGNED_BYTE, raw);

  Tga *t = new Tga();
  t->m_width = width;
  t->m_height = height;
  t->m_texture_id = id;

  return t;
}
