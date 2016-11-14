extern crate openvr_sys;
mod constants;
mod compositor;
mod device;
mod service;

use {VRCompositor, VRService, VRServiceCreator};

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

     fn new_compositor(&self) -> Result<Box<VRCompositor>, String> {
         compositor::OpenVRCompositor::new().map(|c| Box::new(c) as Box<VRCompositor>)
     }
}