#version 450
#include "../bindless.glsl"
#extension GL_ARB_separate_shader_objects : enable


layout(location = 0) in vec3 vPosition;

struct CameraData{
    mat4 viewproj;
    vec3 pos;
};

struct ChunkMesh{
    mat4 model;
};

struct Index{
    uint cam;
    uint chunk;
};






RegisterPushConstant(push, {
    uint cam_index; 
    uint chunkmesh;
});

layout(set = 0, binding = 3) uniform Camera{
    uint camera;
}cam[];


  layout(std430, set = 0, binding = 2) readonly buffer chunk
  {
    ChunkMesh data[];
  }chunks[];

 

RegisterBuffer(readonly, chunkers, ChunkMesh);

RegisterUniform(indices, Index);


void main() {


  //  ChunkMesh chunk = uchunkers[push.chunkmesh].data[gl_InstanceIndex];
    
  //  CameraData camData = cam[index.cam].camera;

    mat4 model = mat4(1.0);
    model[3] = vec4(1.0, 1.0, 1.0, 1.0);

   // outNormal = vNormal;
  //  texCoord = vTexCoord;
  //  outFaceIndex = vFaceIndex;
  //  camPos = camData.pos;
  //  mat_index = object.texture_index;

//    gl_Position = camData.viewproj * model * vec4(vPosition, 1.0);
 //   outFrag = (chunkmesh.model * vec4(vPosition, 1.0)).rgb;
  //  gl_Position = 
}