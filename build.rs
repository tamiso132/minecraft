// build.rs

use std::process::Command;

fn main() {
    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    build_shaders();
}

#[cfg(target_os = "linux")]
fn build_shaders() {
    Command::new("chmod").args(["+x", "build_shader.sh"]);
    Command::new("bash").arg("build_shader.sh").status().unwrap();
}

#[cfg(target_os = "windows")]
fn build_shaders() {
    Command::new(".\\build_shader.bat").status().unwrap();
}
