#version 450 core
#pragma shader_stage(vertex)

layout (push_constant) uniform PushConstants {
	mat4 mvp;
} pc;

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 normal;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec2 uvOut;

void main()
{

	gl_Position = pc.mvp * position;

	uvOut = uv;
}