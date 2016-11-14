use VRCompositor;
use VRDevicePtr;
use VRDisplayEvent;

pub trait VRService: Send {
    fn initialize(&mut self) -> Result<(), String>;

    fn fetch_devices(&mut self) -> Result<Vec<VRDevicePtr>, String>;

    fn is_available(&self) -> bool;

    fn poll_events(&self) -> Vec<VRDisplayEvent>;
}

pub trait VRServiceCreator {
    fn new_service(&self) -> Box<VRService>;
    fn new_compositor(&self) -> Result<Box<VRCompositor>, String>;
}