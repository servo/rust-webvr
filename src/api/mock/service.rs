use {VRService, VRServicePtr, VRDevicePtr, VRDisplayEvent};
use super::device::{MockVRDevice, MockVRDevicePtr};
use std::sync::Arc;
use std::cell::RefCell;

pub struct MockVRService {
    devices: Vec<MockVRDevicePtr>,
    observer: Option<Box<Fn(VRDisplayEvent)>>
}

unsafe impl Send for MockVRService {}

impl VRService for MockVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        Ok(())
    }

    fn fetch_devices(&mut self) -> Result<Vec<VRDevicePtr>,String> {
        if self.devices.len() == 0 {
            self.devices.push(MockVRDevice::new())
        }

        Ok(self.clone_devices())
    }

    fn is_available(&self) -> bool {
        true   
    }

    fn poll_events(&self) {
        // TODO: fake mock events
    }

    fn set_observer(&mut self, callback: Option<Box<Fn(VRDisplayEvent)>>) {
        self.observer = callback;
    }
}

impl MockVRService {
    pub fn new() -> VRServicePtr {
        Arc::new(RefCell::new(MockVRService {
            devices: Vec::new(),
            observer: None
        }))
    }
    fn clone_devices(&self) -> Vec<VRDevicePtr> {
        self.devices.iter().map(|d| d.clone() as VRDevicePtr).collect()
    }
}