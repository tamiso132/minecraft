#!/bin/bash

# Directory containing the GLSL shaders
shader_dir="shaders"

# Create directory for SPIR-V shaders if it doesn't exist
mkdir -p "${shader_dir}/spv"

# Find GLSL shader files and compile them to SPIR-V using glslc
find "$shader_dir" -type f \( -name "*.vert" -o -name "*.frag" -o -name "*.comp" \) -print0 | while IFS= read -r -d '' shader_file; do
    shader_filename=$(basename "$shader_file")
    shader_basename="${shader_filename%.*}"
    shader_extension="${shader_filename##*.}"
    output_file="${shader_dir}/spv/${shader_basename}.${shader_extension}.spv"

    # Compile the shader to SPIR-V using glslc
    glslc "$shader_file" -o "$output_file"
done
