#version 450


layout(location=0) in vec3 i_color;
layout(location=1) in vec2 i_uv;
layout(location=2) in vec2 i_size;
layout(location=3) in float i_is_black;

layout(location=0) out vec4 o_color;

void main() {
    vec3 col = i_color;

    float alpha = 1.0;

    vec2 pos = i_uv * i_size;

    float radius = 5.0;

    float xMax = i_size.x - radius;
    float yMax = i_size.y - radius;

    if (pos.x  < radius && pos.y > yMax ){
        alpha *= 1.0 - smoothstep(radius - 0.7,radius+ 0.7, length(pos - vec2(radius,yMax)));
    }else if (pos.x  > xMax && pos.y > yMax ){
        alpha *= 1.0 - smoothstep(radius - 0.7,radius+ 0.7, length(pos - vec2(xMax,yMax)));
    }

    if(i_is_black==1.0){
        o_color = vec4(col, alpha);
    }else{
        o_color = vec4(col*alpha,1.0);
    }
}