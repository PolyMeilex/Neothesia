#version 330

layout(location = 0) in vec2 v;
layout(location = 1) in vec4 id; //note id, start, duration, is minor

uniform float time;
uniform float mainSpeed;
uniform float minorsWidth = 1.0;

#define notesCount 52.0

out INTERFACE {
	float isMinor;
	vec2 uv;
	vec2 noteSize;
} Out;


void main(){
	
	float scalingFactor = id.w != 0.0 ? minorsWidth : 1.0;
	// Size of the note : width, height based on duration and current speed.
	Out.noteSize = vec2(0.9*2.0/notesCount * scalingFactor, id.z*mainSpeed);
	
	// Compute note shift.
	// Horizontal shift based on note id, width of keyboard, and if the note is minor or not.
	// Vertical shift based on note start time, current time, speed, and height of the note quad.
	const float a = (1.0/(notesCount-1.0)) * (2.0 - 2.0/notesCount);
	const float b = -1.0 + 1.0/notesCount;
	vec2 noteShift = vec2(id.x * a + b + id.w/notesCount, Out.noteSize.y * 0.5 - 0.5 + mainSpeed * (id.y - time));
	
	// Scale uv.
	Out.uv = Out.noteSize * v;
	Out.isMinor = id.w;
	// Output position.
	gl_Position = vec4(Out.noteSize * v + noteShift, 0.0 , 1.0) ;
	
}