#version 330 core
out vec4 FragColor;

in vec3 VertexColor;
in vec2 TexCoord;

uniform sampler2D Texture;

void main() {
    FragColor = texture(Texture, TexCoord);
}