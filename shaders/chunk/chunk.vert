#version 460
#include "../bindless.glsl"
// STRUCTS

struct ChunkConstant {
    uint cam_index;
    vec3 pos;
};

struct CameraData{
    mat4 viewproj;
    vec3 pos;
};

struct Quad{
  uint quad;
};

// Variables

layout(push_constant) uniform constants {
  uint cam_index;
  uint quad_index;
  vec3 chunk_offset;
} push;


layout(set = 0, binding = 3) uniform Camera{
    CameraData camera;
}cam[];

layout(std430, set = 0, binding = 2) readonly buffer Quads{
    Quad quads[];
} quad_buffer[];



const vec3 normalLookup[6] = {
  vec3( 0, 1, 0 ),
  vec3(0, -1, 0 ),
  vec3( 1, 0, 0 ),
  vec3( -1, 0, 0 ),
  vec3( 0, 0, 1 ),
  vec3( 0, 0, -1 )
};

const int flipLookup[6] = int[6](1, -1, -1, 1, -1, 1);

void main(){
  uint quad = quad_buffer[push.quad_index].quads[gl_InstanceIndex].quad;
  CameraData camera = cam[push.cam_index].camera;


  uint mask = (1 << 7) - 1;

  uint x = quad & mask;
  uint y = (quad >> 7) & mask;
  uint z = (quad >> 14) & mask;

  uint w = (quad >> 21) & mask;
  uint h = (quad >> 28) & mask;

  uint face = (quad >> 35) & mask;

  vec3 normal = normalLookup[face / 2];
// todo, get vertex count and do modulus of it.
  gl_Position = camera.viewproj *  vec4(1, 1, 1, 1);
}