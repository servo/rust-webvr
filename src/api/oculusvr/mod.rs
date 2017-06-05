#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

mod display;
mod gamepad;
mod service;
use super::jni_utils;

use {VRService, VRServiceCreator};

pub struct OculusVRServiceCreator;

impl OculusVRServiceCreator {
    pub fn new() -> Box<OculusVRServiceCreator> {
        Box::new(OculusVRServiceCreator)
    }
}

impl VRServiceCreator for OculusVRServiceCreator {
     fn new_service(&self) -> Box<VRService> {
         Box::new(service::OculusVRService::new())
     }
}

// Export functions called from Java
#[cfg(target_os="android")]
pub mod jni {
    pub use super::service::Java_com_rust_webvr_OVRService_nativeOnPause;
    pub use super::service::Java_com_rust_webvr_OVRService_nativeOnResume;
}

