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

