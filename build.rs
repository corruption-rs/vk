extern crate command_macros;

use command_macros::cmd;

fn main() {
    for entry in glob::glob("./assets/shaders/*.vert")
        .expect("Failed to read ./assets/shaders/*.vert")
        .chain(glob::glob("./assets/shaders/*.frag").expect("Failed to read ./assets/shaders/*.frag"))
    {
        match entry {
            Ok(path) => {
                let extension = path
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .expect("Failed to get extension");

                let name = path
                    .file_name()
                    .and_then(std::ffi::OsStr::to_str)
                    .expect("Failed to get file name")
                    .split(".")
                    .collect::<Vec<&str>>();

                let name = name.first().expect("Failed to get file name");

                let location = path
                    .as_os_str()
                    .to_str()
                    .expect("Failed to convert path to str")
                    .split(std::path::MAIN_SEPARATOR)
                    .collect::<Vec<&str>>();

                let location = location.split_last().expect("Failed to split last").1;
                let location = location.join(std::path::MAIN_SEPARATOR.to_string().as_str());

                let mut command = cmd!(glslc 
                    ((String::from("--target-env=vulkan"))) 
                    ((path.to_str().expect("Failed to convert path to string"))) 
                    ((String::from("-o")))
                    ((format!("{}/{}_{}.spv", location, name, extension.split_at(1).0)))
                    ((format!("-fshader-stage={}", extension))));
                command.spawn().expect("Failed to launch glslc, make sure it's in your $PATH");
            }
            Err(e) => panic!("{}", e),
        }
    }
}
