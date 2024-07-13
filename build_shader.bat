@echo off
setlocal EnableDelayedExpansion

REM Directory containing the GLSL shaders
set "shader_dir=shaders"

REM Create directory for SPIR-V shaders if it doesn't exist
if not exist "%shader_dir%\spv" mkdir "%shader_dir%\spv"

REM Find GLSL shader files and compile them to SPIR-V using glslc
for /r "%shader_dir%" %%F in (*.vert *.frag *.comp) do (
    set "shader_file=%%~fF"
    set "shader_filename=%%~nxF"
    set "shader_basename=%%~nF"
    set "shader_extension=%%~xF"
    set "output_file=!shader_dir!\spv\!shader_basename!!shader_extension!.spv"

    REM Compile the shader to SPIR-V using glslc
    glslc "!shader_file!" -o "!output_file!"
)

endlocal