#version 460
#include "../bindless.glsl"
// STRUCTS

#extension GL_ARB_gpu_shader_int64 : enable
#extension GL_ARB_gpu_shader_fp64 : enable


struct ColorOut{
 float r;
 float g;
 float b;
 float a;
};

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


layout(push_constant) uniform constants {
  vec3 chunk_offset;
  uint cam_index;
  uint quad_index;
  uint color_index;
  uint world_index;
  uint chunk_size;
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
const uint flipLookup[6] = uint[6](1, 0, 1, 0, 1, 0);
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

const float voxel_scale = 0.015625;
//const float voxel_scale = 1.0;

const vec3 add_on_flip[] = vec3[6](vec3(0, 0, 0),vec3(1, 0, 0),vec3(0, 0, 0), vec3(0, 0, 1), vec3(0, 0, 0),vec3(0, 1, 0)); 

layout(location = 1) out uint face_num;
layout(location = 2) out vec3 world_pos;


void main(){
  int64_t quad = quad_buffer[push.quad_index].quads[gl_InstanceIndex].quad;

  CameraData camera = cam[push.cam_index].camera;
  int64_t mask = (1 << 7) - 1;

  uint face = uint((quad >> 35) & mask);

  face_num = face;
  uint axis = face/2;

  uint flip_index = flip_axis_index[axis];

  uint flip = flipLookup[face]; 

  float x = float(quad & mask);
  float y = float((quad >> 7) & mask);
  float z = float((quad >> 14) & mask);

  float h = float((quad >> 28) & mask);
  float w = float((quad >> 21) & mask);

// calculate the width axis,  (z, x, x) respective Right, Front, Top
  uint w_dir  = 2 -  2 * (((face >> 2) | (face >> 1)) & 1);

// calculate the height axis,  (y, y, z) respective Right, Front, Top
  uint h_dir = 1 + ((face >> 2) & 1);

// Vertice order depending on the axis
  uvec2 vertex_order =  vertice_orders[gl_VertexIndex + axis * 6];
  
// Toggle vertice bit of specific axis if flipped  
  vertex_order[flip_index] = vertex_order[flip_index] ^ (flip << 0);
float world_w = (w - 1) * float(vertex_order.x);
float world_h = (h - 1) * float(vertex_order.y);

  w *= float(vertex_order.x);
  h *= float(vertex_order.y);



  vec3 adder = add_on_flip[face];

  vec4 final_position = vec4((x + adder.x) * voxel_scale, (y + adder.y) * voxel_scale, (z + adder.z) * voxel_scale, 1.0);


  final_position += (adder, 0);

  final_position[w_dir] += w * voxel_scale;
  final_position[h_dir] += h * voxel_scale;

  world_pos = vec3(x, y, z);
  world_pos[w_dir] += world_w;
  world_pos[h_dir] += world_h;


  vec3 normal = normalLookup[face / 2];
  gl_Position = camera.viewproj * final_position;
}



