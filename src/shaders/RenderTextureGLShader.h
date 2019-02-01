#ifndef RenderTextureGLShader_H
#define RenderTextureGLShader_H

#include <string>

namespace RenderTexture_GLShader {

std::string frag = "#version 330 \n"
                   "in vec2 uv;"
                   "uniform sampler2D screenTexture;"
                   "out vec4 color;"
                   "void main(){"
                   "   color = texture(screenTexture,uv);"
                   "}";
std::string vert = "#version 330 \n "
                   "layout(location = 0) in vec2 v;"
                   "out vec2 uv;"
                   "void main(){"
                   "   gl_Position = vec4(v.xy,0.0f,1.0f);"
                   "   uv = v.xy * 0.5 + 0.5;"
                   "}";
} // namespace RenderTexture_GLShader
#endif