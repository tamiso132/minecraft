#version 450

#include "../bindless.glsl"


struct ColorOut{
 float r;
 float g;
 float b;
 float a;
};

layout (location = 2) in flat ColorOut in_color;
layout(location = 1) in flat uint face;
layout(location = 0) out vec4 finalColor;


const vec3 colors[6] = vec3[6](
    vec3(1, 0, 0),
    vec3(1, 0, 0),
    vec3(0, 1, 0),
    vec3(0, 1, 0),
    vec3(0, 0, 1),
    vec3(0, 0, 1)
);

void main() {
    //finalColor = vec4(0.5, 0.5, 0.5, 1.0);
  //  r: 221, g: 221, b: 221
ColorOut color;
  if((in_color.r == 0.0) || (in_color.g == 0.0) || (in_color.b == 0.0)) {
    color.r = 1.0;
    color.g = 1.0;
    color.b = 1.0;
  }

    finalColor = vec4(color.r, color.g, color.b, 1.0);
}
