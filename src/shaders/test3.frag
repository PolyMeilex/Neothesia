float roundedRectangle(vec2 pos, vec2 size, float radius, vec2 uv) {
  float d = length(max(abs(uv - pos), size) - size) - radius;
  return smoothstep(0.66, 0.33, d / 40.0 * 5.0);
}

void main() {
    // vec2 uv = gl_FragCoord.xy / iResolution.y * 2. - 1.;
    // uv.x -= (iResolution.x - iResolution.y) / iResolution.y;
//   vec2 uv = gl_FragCoord.xy / iResolution.xy;

    vec3 color;

    vec2 size = vec2(100.);
    vec2 pos = vec2(6.0+size.x,iResolution.y-size.y-6.0);

    float rect = roundedRectangle(pos,size,1.0,gl_FragCoord.xy); 
    color += rect;
  

    gl_FragColor = vec4(color, 1.0);
}