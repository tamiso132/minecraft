// build.rs
use std::{ffi::OsStr, fs, path::Path, process::Command};

fn main() {
    // Note that there are a number of downsides to this approach, the comments
    // below detail how to improve the portability of these commands.
    build_shaders();
}

const OUTPUT_DIRECTORY: &str = "shaders/spv";
const SPV_EXT: &str = "spv";

fn build_shaders() {
    let mut error = false;
    read_directory(Path::new("shaders"), &mut error);

    if error {
        panic!("shaders are not compiling correctly");
    }
    println!("cargo:warning=Shaders are compiled");
}

fn read_directory(path: &Path, error: &mut bool) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            read_directory(&path, error);
        } else if path.is_file() {
            let ext = path.extension().unwrap_or(OsStr::new(""));
            if ext == "frag" || ext == "vert" || ext == "comp" {
                let full_filename = path.file_name().unwrap();
                let file_name = &full_filename.as_encoded_bytes()[0..get_filename_index_without_ext(full_filename)];

                let output_directory = format!(
                    "{}/{}.{}.{}",
                    OUTPUT_DIRECTORY,
                    std::str::from_utf8(file_name).unwrap(),
                    ext.to_str().unwrap(),
                    SPV_EXT
                );

                match Command::new("glslc").arg(path.to_str().unwrap()).arg("-o").arg(output_directory).output() {
                    Ok(x) => {
                        let sterr = std::str::from_utf8(x.stderr.trim_ascii_end().trim_ascii_start()).unwrap();
                        if !sterr.is_empty() {
                            println!("Error: {}", sterr);
                            *error = true;
                        }
                    }
                    Err(e) => {
                        panic!("{}", e);
                    }
                }
            }
        }
    }
}

fn get_filename_index_without_ext(filename: &OsStr) -> usize {
    let dot_byte = ".".as_bytes()[0];
    for i in 0..filename.as_encoded_bytes().len() {
        let byte = filename.as_encoded_bytes()[i];
        if byte == dot_byte {
            return i as usize;
        }
    }
    panic!();
}
