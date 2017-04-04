#![cfg(feature = "googlevr")]

mod display;
mod gamepad;
mod service;
mod jni_utils;

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
