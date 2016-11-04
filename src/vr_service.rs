use VRDevicePtr;
use VRDisplayEvent;
use std::sync::Arc;
use std::cell::RefCell;
pub type VRServicePtr = Arc<RefCell<VRService>>;

pub trait VRService: Send {
    fn initialize(&mut self) -> Result<(), String>;

    fn fetch_devices(&mut self) -> Result<Vec<VRDevicePtr>, String>;

    fn is_available(&self) -> bool;

    fn poll_events(&self) -> Vec<VRDisplayEvent>;
}