mod display;
mod service;

pub use {VRService, VRServiceCreator, VREyeParameters, VRStageParameters, MockVRControlMsg, MockVRInit, MockVRView};
use std::sync::mpsc::{channel, Sender};

pub struct MockServiceCreator;

impl MockServiceCreator {
    pub fn new() -> Box<dyn VRServiceCreator> {
        Box::new(MockServiceCreator)
    }

    pub fn new_service_with_remote(init: MockVRInit) -> (Box<dyn VRService>, Sender<MockVRControlMsg>) {
        let (send, rcv) = channel();
        let service = service::MockVRService::new_with_receiver(rcv, init);
        (Box::new(service), send)
    }
}

impl VRServiceCreator for MockServiceCreator {
     fn new_service(&self) -> Box<dyn VRService> {
         Box::new(service::MockVRService::new(Default::default()))
     }
}
