use std::fs::{self, File};
use std::io::prelude::*;
use std::path::Path;

pub fn generate_spirv(path: &Path, name: String, shader_type: glsl_to_spirv::ShaderType) {
    let glsl = fs::read_to_string(&path).unwrap();
    let mut spirv = glsl_to_spirv::compile(&glsl, shader_type).unwrap();

    let mut buffer = Vec::new();
    spirv.read_to_end(&mut buffer).unwrap();

    let spirv_path = path.parent().unwrap().join(name + ".spv");
    let mut file = File::create(spirv_path).unwrap();
    file.write_all(&buffer).unwrap();
}

fn compile(path: &Path) {
    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

        let path = entry.path();

        if path.is_dir() {
            compile(&path);
        } else {
            let name = entry.file_name().into_string().unwrap();

            if name.ends_with(".vert") {
                println!(
                    "cargo:rerun-if-changed={}",
                    path.clone().into_os_string().into_string().unwrap()
                );
                generate_spirv(&path, name, glsl_to_spirv::ShaderType::Vertex);
            } else if name.ends_with(".frag") {
                println!(
                    "cargo:rerun-if-changed={}",
                    path.clone().into_os_string().into_string().unwrap()
                );
                generate_spirv(&path, name, glsl_to_spirv::ShaderType::Fragment);
            }
        }
    }
}

fn main() {
    if cfg!(feature = "compile_shader") {
        println!("cargo:warning=COMPILING_SHADERS");
        let path = Path::new("./src");
        compile(&path);
    }
}
