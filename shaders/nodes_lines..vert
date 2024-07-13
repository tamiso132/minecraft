#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 vPosition;

struct CameraData{
    mat4 viewproj;
    vec3 pos;
};

layout(push_constant) uniform constants {
  uint camera_index;
  uint node_index;
} push;

layout(set = 0, binding = 3) uniform Camera{
    CameraData camera;
}cam[];


void main() {

    Object object = objects[index.object].model[gl_InstanceIndex];
    CameraData camData = cam[index.cam].camera;

    mat4 model = mat4(1.0);
    model[3] = vec4(1.0, 1.0, 1.0, 1.0);

    outNormal = vNormal;
    texCoord = vTexCoord;
    outFaceIndex = vFaceIndex;
    camPos = camData.pos;
    mat_index = object.texture_index;

    gl_Position = camData.viewproj * model * vec4(vPosition, 1.0);
    outFrag = (model * vec4(vPosition, 1.0)).rgb;
    gl_Position = push_constant.ortho*vec4(vPosition.x, vPosition.y, 0.0, 1.0);
}