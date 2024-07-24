#version 450
#include "../bindless.glsl"
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 oColor;

layout(location = 0) out vec4 finalColor;

layout(push_constant) uniform Matrices {
    mat4 ortho;
    uint texture_index;
} push_constant;




void main() {
    finalColor = vec4(0.5f ,0.5f, 0.5f, 1.0f);
}