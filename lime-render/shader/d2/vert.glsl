#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

layout(location = 0) out vec4 v_color;

layout(set = 0, binding = 0) uniform Data {
    uniform vec2 dimensions;
} uniforms;

void main() {
    v_color = color;
    float x = (float(position.x * 2) / uniforms.dimensions.x) - 1.0;
    float y = (float(position.y * 2) / uniforms.dimensions.y) - 1.0;
    gl_Position = vec4(x, y, 0.0, 1.0);
}