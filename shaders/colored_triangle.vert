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
layout(location = 5) out uint mat_index;

struct Index{
  uint cam; // uniform buffer index
  uint object; // storage buffer index
  uint texture; // normal sampler2DArray index
  uint normal; // normal sampler2DArray index
  uint material; // material buffer index
};

struct CameraData{
    mat4 viewproj;
    vec3 pos;
};


struct Object{
    vec3 position;
    uint texture_index;
};

layout(push_constant) uniform constants {
  uint index;
} push;

layout(set = 0, binding = 3) uniform Indices{
  Index index;
}indices[];

layout(set = 0, binding = 3) uniform Camera{
    CameraData camera;
}cam[];


layout(std430, set = 0, binding = 2) readonly buffer Objects{
    Object model[];
} objects[];

void main() {
    Index index = indices[push.index].index;

    Object object = objects[index.object].model[gl_InstanceIndex];
    CameraData camData = cam[index.cam].camera;

    mat4 model = mat4(1.0);
    model[3] = vec4(object.position, 1.0);

    outNormal = vNormal;
    texCoord = vTexCoord;
    outFaceIndex = vFaceIndex;
    camPos = camData.pos;
    mat_index = object.texture_index;

    gl_Position = camData.viewproj * model * vec4(vPosition, 1.0);
    outFrag = (model * vec4(vPosition, 1.0)).rgb;
}
