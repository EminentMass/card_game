#version 450
#pragma shader_stage(fragment)

const int GLOBAL_LIGHT_COUNT = 4;
const int POINT_LIGHT_COUNT = 8;
const int SPOT_LIGHT_COUNT = 8;

const float AMBIENT_STRENGTH = 0.02;

layout (location = 0) in vec2 tex_coord;
layout (location = 1) in vec3 normal_world;
layout (location = 2) in vec3 position_world;

layout (location = 0) out vec4 outFragColor;

layout (set = 0, binding = 0) uniform Camera {
    mat4 projection_view;
    vec3 position;
} cam;

layout (set = 1, binding = 0) uniform texture2D tex;
layout (set = 1, binding = 1) uniform sampler sam;

layout (set = 2, binding = 0) uniform GlobalLights {
    vec3 color;
    float power;
    vec3 direction;
} global_lights[GLOBAL_LIGHT_COUNT];

layout (set = 2, binding = 1) uniform PointLights {
    vec3 position;
    float radius;
    vec3 color;
    float power;
} point_lights[POINT_LIGHT_COUNT];

layout (set = 2, binding = 2) uniform SpotLights {
    vec3 position;
    float radius;
    vec3 color;
    float power;
    vec3 direction;
    float cut_off;
} spot_lights[SPOT_LIGHT_COUNT];



// The current shader does not handle non uniform scaling as normal vectors will not be properly aligned or scaled.
// TODO: update shader to handle non uniform scaling
// TODO: update both shaders fragment and vertex to use view space instead of world
void main()
{
    //vec3 lightPosition = vec3(0.0, 0.0, 0.0);
    //vec3 lightColor = vec3(1.0, 0.5, 0.5);

    vec3 ambient_color = vec3(1.0, 1.0, 1.0) * AMBIENT_STRENGTH;

    vec3 texture_color = texture(sampler2D(tex, sam), tex_coord).xyz;

    vec3 view_dir = normalize(cam.position.xyz - position_world);

    // possible to cut off lights outside their radius but currently it is probably more performant to not have branching in the shader.
    // If more lights are needed or the system is made more complex it would be a good idea to check this over again.
    // Currently you would also have to calculate the world position of each fragment.
    vec3 light_sum = vec3(0.0);

    for (int i=0; i<GLOBAL_LIGHT_COUNT; i++) {
        vec3 light_dir = normalize(-global_lights[i].direction);
        vec3 half_dir = normalize(view_dir + light_dir);

        float specular_strength = pow(max(dot(normal_world, half_dir), 0.0), 32.0);
        vec3 specular_color = specular_strength * global_lights[i].color;

        float diffuse_strength = max(dot(normal_world, light_dir), 0.0);
        vec3 diffuse_color = global_lights[i].color * diffuse_strength;

        light_sum += specular_color + diffuse_color;
    }

    for (int i=0; i<POINT_LIGHT_COUNT; i++) {
        vec3 light_dir = normalize(point_lights[i].position - position_world);
        vec3 half_dir = normalize(view_dir + light_dir);

        float specular_strength = pow(max(dot(normal_world, half_dir), 0.0), 32.0);
        vec3 specular_color = specular_strength * point_lights[i].color;

        float diffuse_strength = max(dot(normal_world, light_dir), 0.0);
        vec3 diffuse_color = point_lights[i].color * diffuse_strength;

        light_sum += specular_color + diffuse_color;
    }

    for (int i=0; i<SPOT_LIGHT_COUNT; i++) {
        vec3 light_dir = normalize(spot_lights[i].position - position_world);

        // TODO: create gradual falloff for lighting
        float theta = dot(light_dir, normalize(-spot_lights[i].direction));
        if( theta > spot_lights[i].cut_off ){
            vec3 half_dir = normalize(view_dir + light_dir);

            float specular_strength = pow(max(dot(normal_world, half_dir), 0.0), 32.0);
            vec3 specular_color = specular_strength * spot_lights[i].color;

            float diffuse_strength = max(dot(normal_world, light_dir), 0.0);
            vec3 diffuse_color = spot_lights[i].color * diffuse_strength;

            light_sum += specular_color + diffuse_color;
        }
    }

    
    outFragColor = vec4((ambient_color + light_sum) * texture_color, 1.0);
}