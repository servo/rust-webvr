extern crate rust_webvr_api;
#[cfg(all(feature = "googlevr", target_os= "android"))]
extern crate gvr_sys;
#[cfg(all(target_os="windows", feature = "openvr"))]
extern crate libloading;
#[macro_use]
extern crate log;
#[cfg(all(feature = "oculusvr", target_os= "android"))]
extern crate ovr_mobile_sys;

#[cfg(any(feature = "googlevr", feature= "oculusvr"))]
mod gl {
    include!(concat!(env!("OUT_DIR"), "/gles_bindings.rs"));
}

pub mod api;
mod vr_manager;

pub use rust_webvr_api::*;
pub use vr_manager::VRServiceManager;
