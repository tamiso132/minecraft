// build.rs

use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    Command::new("chmod").args(["+x", "build_shader.sh"]);

    Command::new("bash").arg("build_shader.sh").status().unwrap();
}
