#ifndef GLSHADER_H
#define GLSHADER_H

#include "OSGraphics.h"
#include <string>


struct GLShader {
  std::string vert;
  std::string frag;
};


extern struct GLShader RenderTextureGLShader;
extern struct GLShader BlurParticleGLShader;

void GLShadersInit();

GLuint LoadShader(GLShader sh);

#endif