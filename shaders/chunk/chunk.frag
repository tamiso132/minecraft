#version 450

#include "../bindless.glsl"

#extension GL_ARB_gpu_shader_fp64 : enable

struct ColorOut{
 float r;
 float g;
 float b;
 float a;
};


layout(push_constant) uniform constants {
  vec3 chunk_offset;
  uint cam_index;
  uint quad_index;
  uint color_index;
  uint world_index;
  uint chunk_size;
} push;

layout(std430, set = 0, binding = 2) readonly buffer Colors{
    uint colors[];
} color_buffer[];

layout(std430, set = 0, binding = 2) readonly buffer World{
    uint voxels[];
} world_buffer[];


layout(location = 2) in vec3 world;
layout(location = 1) in flat uint face;

layout(location = 0) out vec4 finalColor;



int get_world_index(int x, int y, int z, int chunk_size){
   return (x + (z * chunk_size) + (y * chunk_size * chunk_size));
}

vec4 convert_color(uint color){
	float r = float(color & 0xFF) / 255.0;
	float g = float((color >> 8) & 0xFF) / 255.0;
	float b = float((color >> 16) & 0xFF) / 255.0;
	return vec4(r, g, b, 1.0);
}


int convert_value(float value){
  float epsilon = 0.99;  // Small threshold for detecting precision issues
  
  int rounded_value = int(floor(value));

  if (fract(value) > epsilon)
  {
	rounded_value = int(round(value));	
  }
  return  rounded_value;
}




void main() {

    //finalColor = vec4(0.5, 0.5, 0.5, 1.0);
  //  r: 221, g: 221, b: 221
  ivec3 world_uint = ivec3(convert_value(world.x), convert_value(world.y), convert_value(world.z));
  uint chunk_size = push.chunk_size;
  uint world_index = get_world_index(world_uint.x, world_uint.y, world_uint.z, int(push.chunk_size));

  uint material_index = world_buffer[push.world_index].voxels[world_index]; 
  uint color_packed = color_buffer[push.color_index].colors[material_index];
  vec4 color = convert_color(color_packed);

vec3 cc = pow(color.rgb, vec3(1.0/2.2));
finalColor = vec4(cc.r, cc.g, cc.b, 1.0);
// finalColor = vec4(0.5, 0.5, 0.5, 1.0);
}
