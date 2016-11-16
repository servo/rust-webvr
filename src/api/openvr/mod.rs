extern crate openvr_sys;
mod constants;
mod device;
mod service;

use {VRService, VRServiceCreator};

pub struct OpenVRServiceCreator;

impl OpenVRServiceCreator {
    pub fn new() -> Box<VRServiceCreator> {
        Box::new(OpenVRServiceCreator)
    }
}

impl VRServiceCreator for OpenVRServiceCreator {

     fn new_service(&self) -> Box<VRService> {
         Box::new(service::OpenVRService::new())
     }
}