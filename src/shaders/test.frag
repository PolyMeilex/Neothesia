
void main(){
	vec2 st = gl_FragCoord.xy/iResolution.xy;
	float pct = 0.0;

	pct = 1.0 - distance(st.x,0.5)*2.0;

	if(st.y>0.5){
		pct -= distance(st.y,0.5);
	}
	


	vec4 color = vec4(1,0,0,pct);

	gl_FragColor = color;
}