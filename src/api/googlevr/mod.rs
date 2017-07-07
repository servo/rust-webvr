#![cfg(feature = "googlevr")]

mod display;
mod gamepad;
mod service;
#[cfg(target_os = "android")]
use super::jni_utils;

use {VRService, VRServiceCreator};

pub struct GoogleVRServiceCreator;

impl GoogleVRServiceCreator {
    pub fn new() -> Box<GoogleVRServiceCreator> {
        Box::new(GoogleVRServiceCreator)
    }
}

impl VRServiceCreator for GoogleVRServiceCreator {
     fn new_service(&self) -> Box<VRService> {
         Box::new(service::GoogleVRService::new())
     }
}

// Export functions called from Java
#[cfg(target_os="android")]
pub mod jni {
    pub use super::service::Java_com_rust_webvr_GVRService_nativeOnPause;
    pub use super::service::Java_com_rust_webvr_GVRService_nativeOnResume;
}

