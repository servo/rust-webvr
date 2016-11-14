mod device;
mod service;
mod compositor;

use {VRCompositor, VRService, VRServiceCreator};

pub struct MockServiceCreator;

impl MockServiceCreator {
    pub fn new() -> Box<VRServiceCreator> {
        Box::new(MockServiceCreator)
    }
}

impl VRServiceCreator for MockServiceCreator {

     fn new_service(&self) -> Box<VRService> {
         Box::new(service::MockVRService::new())
     }

     fn new_compositor(&self) -> Result<Box<VRCompositor>, String> {
         Ok(Box::new(compositor::MockVRCompositor::new()))
     }
}