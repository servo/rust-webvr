use {VRService, VRDisplayPtr, VREvent, VRGamepadPtr};
use super::display::{MockVRDisplay, MockVRDisplayPtr};

pub struct MockVRService {
    display: MockVRDisplayPtr,
}

unsafe impl Send for MockVRService {}

impl VRService for MockVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        Ok(vec![self.display.clone()])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        true   
    }

    fn poll_events(&self) -> Vec<VREvent> {
        // TODO: fake mock events
        Vec::new()
    }
}

impl MockVRService {
    pub fn new() -> MockVRService {
        MockVRService {
            display: MockVRDisplay::new(),
        }
    }
}