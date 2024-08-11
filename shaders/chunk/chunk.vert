#version 460
#include "../bindless.glsl"
// STRUCTS

#extension GL_ARB_gpu_shader_int64 : enable

struct ChunkConstant {
    uint cam_index;
    vec3 pos;
};

struct CameraData{
    mat4 viewproj;
    vec3 pos;
};

struct Quad{
  int64_t quad;
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
// fliping, 1 means flip
const uint flipLookup[6] = uint[6](0, 1, 0, 1, 0, 1);

// which axis to flip in the vertice order
const int flip_axis_index[3] = int[3](0, 1, 0);

// the winding order, 
const uvec2 vertice_orders[18] = uvec2[18](
    // Right
    uvec2(0, 0),
    uvec2(0, 1),
    uvec2(1, 1),
    uvec2(0, 0),
    uvec2(1, 1),
    uvec2(1, 0),
    // Front
    uvec2(0, 0),
    uvec2(1, 0),
    uvec2(1, 1),
    uvec2(0, 0),
    uvec2(1, 1),
    uvec2(0, 1),
    //Top
    uvec2(1, 0),
    uvec2(0, 0),
    uvec2(0, 1),
    uvec2(1, 0),
    uvec2(0, 1),
    uvec2(1, 1)
);

layout(location = 1) out uint face_num;

const float voxel_scale = 0.1;

void main(){
  int64_t quad = quad_buffer[push.quad_index].quads[gl_InstanceIndex].quad;
  CameraData camera = cam[push.cam_index].camera;
  int64_t mask = (1 << 7) - 1;

  uint face = uint((quad >> 35) & mask);

  face_num = face;
  uint axis = face/2;

  uint flip_index = flip_axis_index[axis];

  uint flip = flipLookup[face]; 


// Get the voxel data from the quad
  float x = float(quad & mask) * voxel_scale;
  float y = float((quad >> 7) & mask) * voxel_scale;
  float z = float((quad >> 14) & mask) * voxel_scale;

  float w = float((quad >> 21) & mask) * voxel_scale;
  float h = float((quad >> 28) & mask) * voxel_scale;

// calculate the width axis,  (z, x, x) respective Right, Front, Top
  uint w_dir  = 2 -  2 * (((face >> 2) | (face >> 1)) & 1);

// calculate the height axis,  (y, y, z) respective Right, Front, Top
  uint h_dir = 1 + ((face >> 2) & 1);

// Vertice order depending on the axis
  uvec2 vertex_order =  vertice_orders[gl_VertexIndex + axis * 6];
  
// Toggle vertice bit of specific axis if flipped  
  vertex_order[flip_index] = vertex_order[flip_index] ^ (flip << 0);

  w *=   float(vertex_order.x);
  h *= float(vertex_order.y);

  vec4 final_position = vec4(x, y, z, 1);
  final_position[w_dir] += w;
  final_position[h_dir] += h;


  vec3 normal = normalLookup[face / 2];
  gl_Position = camera.viewproj * final_position;
}