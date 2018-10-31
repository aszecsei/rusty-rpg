#version 450
layout(location = 0) in vec3 aPosition;
layout(location = 1) in vec3 aColor;
layout(location = 2) in vec2 aTexCoord;

layout(set = 0, binding = 0) uniform Data {
    mat4 projection;
    mat4 view;
    mat4 world;
} uniforms;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec2 texCoord;

void main() {
    gl_Position = uniforms.projection * uniforms.view * uniforms.world * vec4(aPosition, 1.0);
    fragColor = aColor;
    texCoord = aTexCoord;
}