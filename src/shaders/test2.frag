float roundedRectangle (vec2 pos, vec2 size, float radius, float thickness, vec2 uv)
{
  float d = length(max(abs(uv - pos),size) - size) - radius;
  return smoothstep(0.66, 0.33, d / thickness * 5.0);
}

void main(){
  vec2 uv = (gl_FragCoord.xy / iResolution.xx - 0.5) * 8.0;

  

  vec3 rectColor = vec3( sin(iTime), cos(iTime), 0.5);
  float intensity = roundedRectangle(vec2(0.0,0.0), vec2(1,1),0.1,0.1,uv);

  vec3 col;
  col = mix(col,rectColor, intensity);
  gl_FragColor = vec4(col,1.0);
}