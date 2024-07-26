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

const int flipLookup[6] = int[6](1, -1, -1, 1, -1, 1);
const int flip_axis_index[3] = int[3](0, 1, 0);

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
    uvec2(0, 0),
    uvec2(1, 0),
    uvec2(1, 1),
    uvec2(0, 0),
    uvec2(1, 1),
    uvec2(0, 1)
);

layout(location = 1) out vec4 position;
layout(location = 2) out vec2 size;
layout(location = 3) out vec2 instance_and_vertex_id;
layout(location = 4) out uint face_num;

const float voxel_scale = 1;

void main(){
  int64_t quad = quad_buffer[push.quad_index].quads[gl_InstanceIndex].quad;
  CameraData camera = cam[push.cam_index].camera;
  int64_t mask = (1 << 7) - 1;

  uint face = uint((quad >> 35) & mask);

  face_num = face;
  uint axis = face/2;

  uint flip_index = flip_axis_index[axis];

  float flip = float(flipLookup[face]); 


  instance_and_vertex_id = vec2(gl_InstanceIndex, gl_VertexIndex);

  float x = float(quad & mask) * voxel_scale;
  float y = float((quad >> 7) & mask) * voxel_scale;
  float z = float((quad >> 14) & mask) * voxel_scale;

  uint w_dir  = 2 -  2 * (((face >> 2) | (face >> 1)) & 1);
  uint h_dir = 1 + ((face >> 2) & 1);

  uvec2 vertex_order =  vertice_orders[gl_VertexIndex + axis * 6];
  vertex_order[flip_index] =  vertex_order[flip_index] ^ (1 << 0);


  float w = float((quad >> 21) & mask) * float(vertex_order.x) * voxel_scale;
  float h = float((quad >> 28) & mask) * float(vertex_order.y) * voxel_scale;

  vec2 sizes = vec2(w, h);

  

  size = vec2(float((quad >> 21) & mask), float((quad >> 28) & mask));

  vec4 final_position = vec4(x, y, z, 1);

  
  final_position[w_dir] += sizes[0];
  final_position[h_dir] += sizes[1];

  position = final_position;

 

  vec3 normal = normalLookup[face / 2];
  gl_Position = camera.viewproj * final_position;
}