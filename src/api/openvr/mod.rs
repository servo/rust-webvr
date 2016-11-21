mod binding;
mod constants;
mod device;
mod library;
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