#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 vPosition;
layout(location = 1) in vec2 vUV;
layout(location = 2) in vec4 vColor;

layout(push_constant) uniform Matrices {
    mat4 ortho;
    uint texture_index;
} push_constant;

layout(location = 0) out vec4 oColor;
layout(location = 1) out vec2 oUV;

void main() {
    oColor = vColor;
    oUV = vUV;

    gl_Position = push_constant.ortho*vec4(vPosition.x, vPosition.y, 0.0, 1.0);
}