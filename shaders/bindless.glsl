// Add non-uniform qualifier extension here;
// so, we do not add it for all our shaders
// that use bindless resources
#extension GL_EXT_nonuniform_qualifier : enable
#extension GL_EXT_buffer_reference : require

#define Bindless 1

// We always bind the bindless descriptor set
// to set = 0
#define BindlessDescriptorSet 0
// These are the bindings that we defined
// in bindless descriptor layout
#define BindlessCombinedImage 0
#define BindlessStorageImage 1
#define BindlessStorageBuffer 2
#define BindlessUniformBinding 3

#define GetLayoutVariableName(Name) u##Name##Register

// Register uniform
#define RegisterUniform(Name, Struct) \
  layout(set = BindlessDescriptorSet, binding = BindlessUniformBinding) \
      uniform Name Struct \
      GetLayoutVariableName(Name)[]

// Register storage buffer
#define RegisterBuffer(Layout, BufferAccess, Name, Struct) \
  layout(Layout, set = BindlessDescriptorSet, \
         binding = BindlessStorageBuffer) \
  BufferAccess buffer Name Struct GetLayoutVariableName(Name)[]

// Access a specific resource
#define GetResource(Name, Index) \
  GetLayoutVariableName(Name)[Index]

// Register empty resources
// to be compliant with the pipeline layout
// even if the shader does not use all the descriptors
RegisterUniform(DummyUniform, { uint ignore; });
RegisterBuffer(std430, readonly, DummyBuffer, { uint ignore; });

// Register textures
layout(rgba8,set = BindlessDescriptorSet, binding = BindlessStorageImage) \
uniform image2D globalImages[];

// Register textures
layout(set = BindlessDescriptorSet, binding = BindlessCombinedImage) \
    uniform sampler2D globalSamples[];