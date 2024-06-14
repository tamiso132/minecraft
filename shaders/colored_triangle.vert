#version 460
#include "bindless.glsl"
layout(location = 0) in vec3 vPosition;
layout(location = 1) in vec3 vNormal;
layout(location = 2) in vec2 vTexCoord;
layout(location = 3) in uint vFaceIndex;

layout(location = 0) out vec3 outNormal;
layout(location = 1) out vec2 texCoord;
layout(location = 2) out uint outFaceIndex;
layout(location = 3) out vec3 outFrag;
layout(location = 4) out vec3 camPos;
layout(location = 5) out uint texture_index;


struct CameraData{
    mat4 viewproj;
    vec3 pos;
};

struct Object{
    mat4 model;
    uint texture_index;
};

layout(push_constant) uniform push {
   uint cam;
uint object;
} indices;

layout(set = 0, binding = 3) uniform Camera{
    CameraData camera;
}cam[];


layout(std430, set = 0, binding = 2) readonly buffer Objects{
    Object model[];
} objects[];



void main() {

    Object model = objects[indices.object].model[gl_InstanceIndex];
    CameraData camData = cam[indices.cam].camera;

    outNormal = vNormal;
    texCoord = vTexCoord;
    outFaceIndex = vFaceIndex;
    camPos = camData.pos;
    texture_index = model.texture_index;

    gl_Position = camData.viewproj * model.model * vec4(vPosition, 1.0);
    outFrag = (model.model * vec4(vPosition, 1.0)).rgb;
}
