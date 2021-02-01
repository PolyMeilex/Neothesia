#version 450


layout(location=0) in vec3 i_color;
layout(location=1) in vec2 i_uv;
layout(location=2) in vec2 i_size;
layout(location=3) in float i_radius;

layout(location=0) out vec4 o_color;

layout(set=0, binding=0) 
uniform Uniforms {
    mat4 u_Transform;
    vec2 u_size;
};

void main() {
    vec3 col = i_color;
    
    float keyboard_height = u_size.y / 5.0;
    if(gl_FragCoord.y > u_size.y - keyboard_height){
        discard;
    }

    float alpha = 1.0;

    vec2 pos = i_uv * i_size;

    float xMax = i_size.x - i_radius;
    float yMax = i_size.y - i_radius;

    if (pos.x  < i_radius && pos.y < i_radius ){
        alpha *= 1.0 - smoothstep(i_radius - 0.7,i_radius+ 0.7, length(pos - vec2(i_radius,i_radius)));
    }else if (pos.x  < i_radius && pos.y > yMax ){
        alpha *= 1.0 - smoothstep(i_radius - 0.7,i_radius+ 0.7, length(pos - vec2(i_radius,yMax)));
    }else if (pos.x  > xMax && pos.y > yMax ){
        alpha *= 1.0 - smoothstep(i_radius - 0.7,i_radius+ 0.7, length(pos - vec2(xMax,yMax)));
    }else if (pos.x  > xMax && pos.y < i_radius ){
        alpha *= 1.0 - smoothstep(i_radius - 0.7,i_radius+ 0.7, length(pos - vec2(xMax,i_radius)));
    }

    o_color = vec4(col, alpha);
}