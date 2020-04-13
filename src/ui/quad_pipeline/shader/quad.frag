#version 450


layout(location=0) in vec3 color;
layout(location=1) in vec2 uv;
layout(location=2) in vec2 size;
layout(location=3) in float radius;

layout(location=0) out vec4 f_color;




void main() {
    vec3 col = color;

    float alpha = 1.0;

    // float radius = 15.0;

    vec2 pos = uv * size;

    float xMax = size.x - radius;
    float yMax = size.y - radius;


    if (pos.x  < radius && pos.y < radius ){
        alpha *= 1.0 - smoothstep(radius - 0.7,radius+ 0.7, length(pos - vec2(radius,radius)));
    }else if (pos.x  < radius && pos.y > yMax ){
        alpha *= 1.0 - smoothstep(radius - 0.7,radius+ 0.7, length(pos - vec2(radius,yMax)));
    }else if (pos.x  > xMax && pos.y > yMax ){
        alpha *= 1.0 - smoothstep(radius - 0.7,radius+ 0.7, length(pos - vec2(xMax,yMax)));
    }else if (pos.x  > xMax && pos.y < radius ){
        alpha *= 1.0 - smoothstep(radius - 0.7,radius+ 0.7, length(pos - vec2(xMax,radius)));
    }
    f_color = vec4(col, alpha);
}