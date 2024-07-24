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

}