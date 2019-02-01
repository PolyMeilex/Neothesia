#ifndef BlurParticleGLShader_H
#define BlurParticleGLShader_H

#include <string>

namespace BlurParticle_GLShader {

std::string frag =
    "#version 330 \n"
    "in vec2 uv;"
    "uniform sampler2D screenTexture;"
    "uniform vec2 inverseScreenSize;"
    "uniform vec3 backgroundColor = vec3(0.0, 0.0, 0.0);"
    "uniform float attenuationFactor = 0.99;"
    "out vec4 fragColor;"
    "void main(){"
    "   vec3 color = texture(screenTexture, uv).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(-2,-2)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(-2,2)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(-1,0)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(0,-1)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(0,1)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(1,0)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(2,-2)).rgb;"
    "   color += textureOffset(screenTexture, uv, 2*ivec2(2,2)).rgb;"
    "   fragColor = vec4(mix(backgroundColor, color/9.0, attenuationFactor),1);"
    "}";
std::string vert = "#version 330 \n "
                   "layout(location = 0) in vec2 v;"
                   "out vec2 uv;"
                   "void main(){"
                   "   gl_Position = vec4(v, 0.0, 1.0);"
                   "   uv = v.xy * 0.5 + 0.5;"
                   "}";
} // namespace BlurParticle_GLShader
#endif