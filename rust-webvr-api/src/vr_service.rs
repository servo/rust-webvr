use VRDisplayPtr;
use VREvent;
use VRGamepadPtr;

pub trait VRService: Send {
    fn initialize(&mut self) -> Result<(), String>;

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>, String>;

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String>;

    fn is_available(&self) -> bool;

    fn poll_events(&self) -> Vec<VREvent>;
}

pub trait VRServiceCreator {
    fn new_service(&self) -> Box<dyn VRService>;
}
