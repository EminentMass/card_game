#version 450 core
#pragma shader_stage(vertex)

layout (push_constant) uniform PushConstants {
	mat4 view_projection;
} pc;

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 normal;
layout (location = 2) in vec2 uv;

// TODO: move model matrix to push constants or a bind group.
layout (location = 3) in vec4 model1;
layout (location = 4) in vec4 model2;
layout (location = 5) in vec4 model3;
layout (location = 6) in vec4 model4;

layout (location = 0) out vec2 uvOut;

void main()
{

	mat4 model = mat4(model1, model2, model3, model4);

	vec4 pos = pc.view_projection * model * position;
	gl_Position = pos;

	uvOut = uv;
}