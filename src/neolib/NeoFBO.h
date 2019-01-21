#ifndef __NEOFBO_H
#define __NEOFBO_H

#include "OSGraphics.h"

class NeoFBO {

public:
  NeoFBO(int width_, int height_);

  void Bind();
  void Unbind();

  GLuint GetTexture() { return TextureId; }
private:
  GLuint FboId;
  GLuint TextureId;

  int width;
  int height;
};

#endif