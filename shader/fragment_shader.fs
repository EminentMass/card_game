#version 450
#pragma shader_stage(fragment)

layout (binding = 0) uniform texture2D tex;
layout (binding = 1) uniform sampler sam;

layout (location = 0) in vec2 texCoord;

layout (location = 0) out vec4 outFragColor;

void main()
{
    outFragColor = texture(sampler2D(tex, sam), texCoord);
    //outFragColor = vec4(1.0, 0.0, 0.0, 1.0);
}