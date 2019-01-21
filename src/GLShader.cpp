#include "GLShader.h"

#include <string>
#include <iostream>
#include <fstream>
#include <vector>
#include <algorithm>


struct GLShader BlurParticleGLShader;
struct GLShader RenderTextureGLShader;

void GLShadersInit(){
    BlurParticleGLShader.frag = "#version 330 \n"
                        "in INTERFACE {"
	                        "vec2 uv;"
                        "} In ;"
                        "uniform sampler2D screenTexture;"
                        "uniform vec2 inverseScreenSize;"
                        "uniform vec3 backgroundColor = vec3(0.0, 0.0, 0.0);"
                        "uniform float attenuationFactor = 0.99;"
                        "out vec4 fragColor;"
                        "void main(){"
                        "   vec3 color = texture(screenTexture, In.uv).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(-2,-2)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(-2,2)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(-1,0)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(0,-1)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(0,1)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(1,0)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(2,-2)).rgb;"
                        "   color += textureOffset(screenTexture, In.uv, 2*ivec2(2,2)).rgb;"
                        "   fragColor = vec4(mix(backgroundColor, color/9.0, attenuationFactor),1);"
                        "}";
    BlurParticleGLShader.vert =  "#version 330 \n "
                        "layout(location = 0) in vec3 v;"
                        "out INTERFACE {"
	                    "   vec2 uv;"
                        "} Out ;"
                        "void main(){"
                        "   gl_Position = vec4(v, 1.0);"
                        "   Out.uv = v.xy * 0.5 + 0.5;"
                        "}";


    RenderTextureGLShader.frag = "#version 330 \n"
                        "in vec2 uv;"
                        "uniform sampler2D screenTexture;"
                        "out vec4 color;"
                        "void main(){"
                        "   color = texture(screenTexture,uv);"
                        "}";
    RenderTextureGLShader.vert =  "#version 330 \n "
                        "layout(location = 0) in vec3 v;"
                        "out vec2 uv;"
                        "void main(){"
                        "   gl_Position = vec4(v.x,v.y,0.0f,1.0f);"
                        "   uv = v.xy * 0.5 + 0.5;"
                        "}";
}

GLuint LoadShader(GLShader sh) {
    GLuint vertShader = glCreateShader(GL_VERTEX_SHADER);
    GLuint fragShader = glCreateShader(GL_FRAGMENT_SHADER);

    const char *vertShaderSrc = sh.vert.c_str();
    const char *fragShaderSrc = sh.frag.c_str();

    GLint result = GL_FALSE;
    int logLength;

    // Compile vertex shader
    std::cout << "Compiling vertex shader." << std::endl;
    glShaderSource(vertShader, 1, &vertShaderSrc, NULL);
    glCompileShader(vertShader);

    // Check vertex shader
    glGetShaderiv(vertShader, GL_COMPILE_STATUS, &result);
    glGetShaderiv(vertShader, GL_INFO_LOG_LENGTH, &logLength);
    std::vector<char> vertShaderError((logLength > 1) ? logLength : 1);
    glGetShaderInfoLog(vertShader, logLength, NULL, &vertShaderError[0]);
    std::cout << &vertShaderError[0] << std::endl;

    // Compile fragment shader
    std::cout << "Compiling fragment shader." << std::endl;
    glShaderSource(fragShader, 1, &fragShaderSrc, NULL);
    glCompileShader(fragShader);

    // Check fragment shader
    glGetShaderiv(fragShader, GL_COMPILE_STATUS, &result);
    glGetShaderiv(fragShader, GL_INFO_LOG_LENGTH, &logLength);
    std::vector<char> fragShaderError((logLength > 1) ? logLength : 1);
    glGetShaderInfoLog(fragShader, logLength, NULL, &fragShaderError[0]);
    std::cout << &fragShaderError[0] << std::endl;

    std::cout << "Linking program" << std::endl;
    GLuint program = glCreateProgram();
    glAttachShader(program, vertShader);
    glAttachShader(program, fragShader);
    glLinkProgram(program);

    glGetProgramiv(program, GL_LINK_STATUS, &result);
    glGetProgramiv(program, GL_INFO_LOG_LENGTH, &logLength);
    std::vector<char> programError( (logLength > 1) ? logLength : 1 );
    glGetProgramInfoLog(program, logLength, NULL, &programError[0]);
    std::cout << &programError[0] << std::endl;

    glDeleteShader(vertShader);
    glDeleteShader(fragShader);

    return program;
}