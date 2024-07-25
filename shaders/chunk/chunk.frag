#version 450

#include "../bindless.glsl"




layout(location = 0) out vec4 finalColor;

layout(push_constant) uniform Matrices {
    mat4 ortho;
    uint texture_index;
} push_constant;


void main() {
    finalColor = vec4(1.0, 1.0, 1.0, 1.0);
}