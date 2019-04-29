#[cfg(all(feature = "vrexternal", target_os= "android"))]
extern crate libc;
extern crate rust_webvr_api;
#[cfg(all(feature = "googlevr", target_os= "android"))]
extern crate gvr_sys;
#[cfg(all(target_os="windows", feature = "openvr"))]
extern crate libloading;
#[macro_use]
extern crate log;
#[cfg(all(feature = "oculusvr", target_os= "android"))]
extern crate ovr_mobile_sys;
#[cfg(feature = "glwindow")]
extern crate euclid;
#[cfg(feature = "glwindow")]
extern crate gleam;
#[cfg(feature = "glwindow")]
extern crate glutin;
#[cfg(feature = "serde-serialization")]
#[macro_use] extern crate serde_derive;

#[cfg(any(feature = "googlevr", feature= "oculusvr"))]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gles_bindings.rs"));
}

#[cfg(feature= "oculusvr")]
mod egl {
    #![allow(non_camel_case_types, non_snake_case)]
    use std::os::raw::{c_long, c_void};
    pub type khronos_utime_nanoseconds_t = khronos_uint64_t;
    pub type khronos_uint64_t = u64;
    pub type khronos_ssize_t = c_long;
    pub type EGLint = i32;
    pub type EGLNativeDisplayType = *const c_void;
    pub type EGLNativePixmapType = *const c_void;
    pub type EGLNativeWindowType = *const c_void;
    pub type NativeDisplayType = EGLNativeDisplayType;
    pub type NativePixmapType = EGLNativePixmapType;
    pub type NativeWindowType = EGLNativeWindowType;
    include!(concat!(env!("OUT_DIR"), "/egl_bindings.rs"));
}

#[cfg(feature= "glwindow")]
pub use api::GlWindowVRService;

pub mod api;
mod vr_manager;

pub use rust_webvr_api::*;
pub use vr_manager::VRServiceManager;
