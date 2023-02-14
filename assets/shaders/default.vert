#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 fragColor;

layout(binding = 0) uniform Camera {
    mat4 model;
    mat4 view;
    mat4 proj;
} camera;

void main() {
    gl_Position = camera.proj * camera.view * camera.model * vec4(inPosition, 0.0, 1.0);
    fragColor = inColor;
}
