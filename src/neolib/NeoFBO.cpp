#include "NeoFBO.h"

NeoFBO::NeoFBO(int width_, int height_) : width(width_),height(height_){
  glGenFramebuffers(1, &FboId);
  glBindFramebuffer(GL_FRAMEBUFFER, FboId);

    glGenTextures(1, &TextureId);
    glBindTexture(GL_TEXTURE_2D, TextureId);
      glTexImage2D(GL_TEXTURE_2D, 0, GL_RGB, width, height, 0, GL_RGB, GL_UNSIGNED_BYTE, 0);
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
      glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);

    glFramebufferTexture(GL_FRAMEBUFFER,GL_COLOR_ATTACHMENT0,TextureId,0);

    GLenum drawBuffers[1] = {GL_COLOR_ATTACHMENT0};
    glDrawBuffers(1, drawBuffers);

  glBindFramebuffer(GL_FRAMEBUFFER, 0);
}

void NeoFBO::Bind(){
	glBindFramebuffer(GL_FRAMEBUFFER, FboId);
}

void NeoFBO::Unbind(){
	glBindFramebuffer(GL_FRAMEBUFFER, 0);
}