use {VRService, VRDisplayPtr, VRDisplayEvent, VRGamepadPtr};
use super::display::{MockVRDisplay, MockVRDisplayPtr};

pub struct MockVRService {
    displays: Vec<MockVRDisplayPtr>,
}

unsafe impl Send for MockVRService {}

impl VRService for MockVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        if self.displays.len() == 0 {
            self.displays.push(MockVRDisplay::new())
        }

        Ok(self.clone_displays())
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        true   
    }

    fn poll_events(&self) -> Vec<VRDisplayEvent> {
        // TODO: fake mock events
        Vec::new()
    }
}

impl MockVRService {
    pub fn new() -> MockVRService {
        MockVRService {
            displays: Vec::new(),
        }
    }
    fn clone_displays(&self) -> Vec<VRDisplayPtr> {
        self.displays.iter().map(|d| d.clone() as VRDisplayPtr).collect()
    }
}