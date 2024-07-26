#version 450

#include "../bindless.glsl"


layout(location = 1) in vec4 position;

layout(location = 0) out vec4 finalColor;

layout(location = 4) in flat uint face;


const vec3 colors[6] = vec3[6](
    vec3(1, 0, 0),
    vec3(1, 0, 0),
    vec3(0, 1, 0),
    vec3(0, 1, 0),
    vec3(0, 0, 1),
    vec3(0, 0, 1)
);

void main() {
    finalColor = vec4(colors[face].rgb, 1.0);
}