#version 460 core

layout (push_constant) uniform PushConstants {
	mat4 mvp;
} pc;

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 normal;
layout (location = 2) in vec2 uv;

layout (location = 0) out vec3 color;

void main()
{
	vec4 pos = pc.mvp * position;
	gl_Position = pos;

	vec3 colors[4] = vec3[4](vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0), vec3(1.0, 1.0, 1.0));

	color = colors[gl_VertexIndex];
}