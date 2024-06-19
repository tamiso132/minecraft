#version 460
#include "bindless.glsl"
// shader input
layout(location = 0) in vec3 inNormal;
layout(location = 1) in vec2 inTexCoord;
layout(location = 2) flat in uint inFaceIndex;
layout(location = 3) in vec3 inFragPos;
layout(location = 4) in vec3 camPos;

struct Index{
  uint cam; // uniform buffer index
  uint object; // storage buffer index
  uint texture; // normal sampler2DArray index
  uint normal; // normal sampler2DArray index
  uint material; // material buffer index
};

layout(push_constant) uniform constants {
  uint index;
} push;

layout(set = 0, binding = 3) uniform Indices{
  Index index;
}indices[];


layout(set = 0, binding = 0)  uniform sampler2DArray samplerArray[];

layout(location = 0) out vec4 outFragColor;


void main() {

  Index index = indices[push.index].index;

 // sampler2DArray textureArray =  samplerArray[1];
 // sampler2DArray normalTextureArray = samplerArray[index.normal];
  

    vec4 color = texture(samplerArray[index.texture], vec3(inTexCoord, 3));

   // vec3 lightColor = vec3(1.0f, 1.0f, 1.0f);
  //  vec3 lightPos = vec3(1, 1, 0);

   // vec3 normal = texture(normalTextureArray, vec3(inTexCoord, materialBuffer.materials[0].faceIndices[inFaceIndex])).rgb;

    // transform normal vector to range [-1,1]
  //  normal = normalize(normal * 2.0 - 1.0);

  //  vec3 ambient = lightColor * materialBuffer.materials[0].ambient* color.rgb;

    // diff
  //  vec3 lightDir = normalize(lightPos - inFragPos);
   // float diff = max(dot(normal, lightDir), 0.0);
  //  vec3 diffuse = diff * lightColor * color.rgb * materialBuffer.materials[0].diffuse;

    // specular
   // vec3 viewDir = normalize(camPos - inFragPos);
 //   vec3 halfwayDir = normalize(lightDir + viewDir);
 //   float spec = pow(max(dot(normal, halfwayDir), 0.0), materialBuffer.materials[0].shininess);
 //   vec3 specular = lightColor * (spec * materialBuffer.materials[0].specular);

  //  float distance = length(lightPos - inFragPos);
 //   float attenuation = 5.0 / (distance * distance);

 //   diffuse *= attenuation;
 //   specular *= attenuation;

    // Gamma correction
 //   float gamma = 2.2;

 //   vec3 finalColor = pow(diffuse + ambient + specular, vec3(1.0 / gamma));

    outFragColor = vec4(color.rgb, 1.0f);
}