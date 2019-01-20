#include "GLShader.h"

#include <string>
#include <iostream>
#include <fstream>
#include <vector>
#include <algorithm>


std::string readFile(const char *filePath) {
    std::string content;
    std::ifstream fileStream(filePath, std::ios::in);

    if(!fileStream.is_open()) {
        std::cerr << "Could not read file " << filePath << ". File does not exist." << std::endl;
        return "";
    }

    std::string line = "";
    while(!fileStream.eof()) {
        std::getline(fileStream, line);
        content.append(line + "\n");
    }

    fileStream.close();
    return content;
}

// std::string frag1 = "#version 330 \n in INTERFACE { vec2 uv; } In; uniform sampler2D screenTexture; out vec3 fragColor;void main(){ vec3 color=texture(screenTexture,In.uv).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(-2,-2)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(-2,2)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(-1,0)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(0,-1)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(0,1)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(1,0)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(2,-2)).rgb;color+=textureOffset(screenTexture,In.uv,2*ivec2(2,2)).rgb;fragColor=mix(vec3(0.0,0.0,0.0),color/9.0,0.99);}";
// std::string frag1 = "#version 400 \n "
//                     "layout(location = 0) out vec4 color; \n"
//                     "void main(){color = vec4(0.5,0.0,1.0,1.0);}";

// std::string frag1 = "#version 400 \n"
//                     "out vec4 color;"
//                     "in vec2 TexCoords;"
//                     "uniform sampler2D text;"
//                     "void main(){"
// 	                "color = texture(text, TexCoords);"
//                     "color.a = 0.5;"
// 	                "}";

// std::string vet1 =  "#version 400 \n "
//                     "layout(location = 0) in vec3 position;"
//                     "layout(location = 1) in vec2 texCoords;"
//                     "out vec2 TexCoords;"
//                     "void main(){"
//                     "gl_Position = vec4( position.x, position.y, 0f, 1f);"
//                     "TexCoords = texCoords;"
//                     "}";

// texture(screenTexture,In.uv).rgb;
std::string frag1 = "#version 330 \n"
                    "in INTERFACE {"
                    "   vec2 uv;"
                    "} In ;"
                    "uniform sampler2D screenTexture;"
                    "out vec4 color;"
                    "void main(){"
	                "   color = texture(screenTexture,In.uv);"
                    "color.a = 0.5;"
	                "}";

std::string vet1 =  "#version 330 \n "
                    "layout(location = 0) in vec3 v;"
                    "out INTERFACE {"
                    "   vec2 uv;"
                    "} Out ;"
                    "void main(){"
                    "   gl_Position = gl_Position = vec4(v, 1.0);"
                    "   Out.uv = v.xy * 0.5 + 0.5;"
                    "}";

GLuint LoadShader(const char *vertex_path, const char *fragment_path) {
    GLuint vertShader = glCreateShader(GL_VERTEX_SHADER);
    GLuint fragShader = glCreateShader(GL_FRAGMENT_SHADER);

    // Read shaders
    std::string vertShaderStr = vet1;
    std::string fragShaderStr = frag1;
    const char *vertShaderSrc = vertShaderStr.c_str();
    const char *fragShaderSrc = fragShaderStr.c_str();

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