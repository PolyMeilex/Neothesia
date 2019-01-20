#iChannel0 https://66.media.tumblr.com/tumblr_mcmeonhR1e1ridypxo1_500.jpg

void main(){
	vec2 uv = (gl_FragCoord.xy / vec2(100,100));
	vec4 color = texture2D(iChannel0,uv);
	gl_FragColor = color;
}