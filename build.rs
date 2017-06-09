extern crate gl_generator;

use std::env;
use std::fs::File;
use std::path::Path;
use gl_generator::{Registry, Api, Profile, Fallbacks};

fn main() {

    if !cfg!(feature = "googlevr") && !cfg!(feature = "oculusvr")  {
        return;
    }

    let dest = env::var("OUT_DIR").unwrap();

    let mut file = File::create(&Path::new(&dest).join("gles_bindings.rs")).unwrap();

    // GLES 2.0 bindings
    let gles_reg = Registry::new(Api::Gles2, (3, 0), Profile::Core, Fallbacks::All, []);
    gles_reg.write_bindings(gl_generator::StaticGenerator, &mut file)
            .unwrap();
}
