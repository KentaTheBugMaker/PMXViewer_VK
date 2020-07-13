#version 450

layout(location = 0) in vec4 position;
layout(location = 1) in vec4 normal;
layout(location = 2) in vec2 uv;
layout(location = 0) out vec4 v_normal;
layout(location = 1) out vec2 v_tex_coord;
layout(location = 2) out vec3 v_position;
layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;


void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    v_normal = transpose(inverse(worldview)) * normal;
    v_tex_coord = uv;
    gl_Position = uniforms.proj * worldview * position;
    v_position=gl_Position.xyz/gl_Position.w;
}