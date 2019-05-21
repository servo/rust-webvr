mod display;
mod service;

use {VRService, VRServiceCreator, VREyeParameters, VRStageParameters};
use std::sync::mpsc::{channel, Sender};

pub struct MockServiceCreator;

impl MockServiceCreator {
    pub fn new() -> Box<VRServiceCreator> {
        Box::new(MockServiceCreator)
    }

    pub fn new_service_with_remote() -> (Box<VRService>, Sender<MockVRControlMsg>) {
        let (send, rcv) = channel();
        let service = service::MockVRService::new_with_receiver(rcv);
        (Box::new(service), send)
    }
}

impl VRServiceCreator for MockServiceCreator {
     fn new_service(&self) -> Box<VRService> {
         Box::new(service::MockVRService::new())
     }
}

pub enum MockVRControlMsg {
    SetViewerPose([f32; 3], [f32; 4]),
    SetEyeParameters(VREyeParameters, VREyeParameters),
    SetProjectionMatrices([f32; 16], [f32; 16]),
    SetStageParameters(VRStageParameters),
}
