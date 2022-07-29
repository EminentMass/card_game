#version 450 core
#pragma shader_stage(vertex)

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 normal;
layout (location = 2) in vec2 tex_coord;

layout (set = 0, binding = 0) uniform Camera {
    mat4 projection_view;
	vec3 position;
} cam;

layout (push_constant) uniform PushConstants {
	mat4 model;
} pc;

layout (location = 0) out vec2 tex_coord_out;
layout (location = 1) out vec3 normal_world;
layout (location = 2) out vec3 position_world;

void main()
{
	mat4 mvp = cam.projection_view * pc.model;
	gl_Position = mvp * position;

	tex_coord_out = tex_coord;
	normal_world = (pc.model * normal).xyz;
	position_world = (pc.model * position).xyz;

}