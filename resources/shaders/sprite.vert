#version 450
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;
layout(location = 2) in vec2 tex_coord;

layout(set = 1, binding = 0) uniform Data {
    mat4 projection;
    mat4 view;
    mat4 world;
} uniforms;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 texCoord;

void main() {
    gl_Position = uniforms.projection * uniforms.view * uniforms.world * vec4(position, 1.0);
    fragColor = color;
    texCoord = tex_coord;
}