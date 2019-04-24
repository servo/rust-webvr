extern crate bindgen;
extern crate gl_generator;

use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use gl_generator::{Registry, Api, Profile, Fallbacks};

fn main() {
    // Copy AARs
    if let Ok(aar_out_dir) = env::var("AAR_OUT_DIR") {
        fs::copy(&Path::new("src/api/googlevr/aar/GVRService.aar"),
                 &Path::new(&aar_out_dir).join("GVRService.aar")).unwrap();

        fs::copy(&Path::new("src/api/oculusvr/aar/OVRService.aar"),
                 &Path::new(&aar_out_dir).join("OVRService.aar")).unwrap();
    }

    if !cfg!(feature = "googlevr") && !cfg!(feature = "oculusvr") && !cfg!(feature = "vrexternal") && !cfg!(feature = "magicleap") {
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();

    // GLES 2.0 bindings
    let mut file = File::create(&Path::new(&out_dir).join("gles_bindings.rs")).unwrap();
    let gles_reg = Registry::new(Api::Gles2, (3, 0), Profile::Core, Fallbacks::All, [
        "GL_OVR_multiview2", "GL_OVR_multiview", "GL_OVR_multiview_multisampled_render_to_texture"]);
    gles_reg.write_bindings(gl_generator::StaticGenerator, &mut file)
            .unwrap();

    // EGL bindings
    if cfg!(any(feature = "magicleap", feature = "oculusvr")) {
        let mut file = File::create(&Path::new(&out_dir).join("egl_bindings.rs")).unwrap();
        Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, ["EGL_KHR_fence_sync"])
            .write_bindings(gl_generator::StaticGenerator, &mut file).unwrap();
        println!("cargo:rustc-link-lib=EGL");
    }

    // Magicleap C API
    if cfg!(feature = "magicleap") {
        let mut builder = bindgen::Builder::default()
            .header("src/api/magicleap/magicleap_c_api.h")
            .blacklist_type("MLResult")
            .derive_default(true)
            .rustfmt_bindings(true);

        if let Ok(mlsdk) = env::var("MAGICLEAP_SDK") {
            builder = builder.clang_args(&[
                format!("--no-standard-includes"),
                format!("--sysroot={}", mlsdk),
                format!("-I{}/include", mlsdk),
                format!("-I{}/lumin/usr/include", mlsdk),
                format!("-I{}/tools/toolchains/lib64/clang/3.8/include", mlsdk),
            ]);
        }

        if let Ok(flags) = env::var("CFLAGS") {
            for flag in flags.split_whitespace() {
                builder = builder.clang_arg(flag);
            }
        }

        if let Ok(flags) = env::var("CLANGFLAGS") {
            for flag in flags.split_whitespace() {
                builder = builder.clang_arg(flag);
            }
        }

        let bindings = builder.generate().expect("Unable to generate bindings");
        let out_path = PathBuf::from(&out_dir);
        bindings.write_to_file(out_path.join("magicleap_c_api.rs"))
            .expect("Couldn't write bindings!");
    }

    let target = env::var("TARGET").unwrap();
    if target.contains("android") && cfg!(feature = "vrexternal") {
        let mut builder = bindgen::Builder::default()
            .header("src/api/vrexternal/cpp/moz_external_vr.h")
            .clang_args(&["-x", "c++", "-std=gnu++11"])
            .whitelist_type("mozilla::gfx::VRExternalShmem")
            .disable_name_namespacing()
            .derive_default(true)
            .rustfmt_bindings(true);

        if let Ok(flags) = env::var("CXXFLAGS") {
            for flag in flags.split_whitespace() {
                builder = builder.clang_arg(flag);
            }
        }

        if let Ok(flags) = env::var("CLANGFLAGS") {
            for flag in flags.split_whitespace() {
                builder = builder.clang_arg(flag);
            }
        }

        let bindings = builder.generate().expect("Unable to generate bindings");
        let out_path = PathBuf::from(&out_dir);
        bindings.write_to_file(out_path.join("moz_external_vr.rs"))
            .expect("Couldn't write bindings!");
    }

}
