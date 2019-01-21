use super::display::{VRExternalDisplay, VRExternalDisplayPtr};
use super::VRExternalShmemPtr;
use {VRDisplayPtr, VREvent, VRGamepadPtr, VRService};

pub struct VRExternalService {
    shmem: VRExternalShmemPtr,
    display: Option<VRExternalDisplayPtr>,
}

unsafe impl Send for VRExternalService {}

impl VRService for VRExternalService {
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>, String> {
        if self.display.is_none() {
            // Block until enumerationCompleted is true.
            self.shmem.as_mut().pull_system(&|state| state.enumerationCompleted);
            let display = VRExternalDisplay::new(self.shmem.clone());
            self.display = Some(display);
        }
        Ok(vec![self.display.as_ref().unwrap().clone()])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String> {
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        true
    }

    fn poll_events(&self) -> Vec<VREvent> {
        match &self.display {
            None => vec![],
            Some(display) => {
                display.borrow_mut()
                       .poll_events()
                       .into_iter()
                       .map(|e| VREvent::Display(e))
                       .collect()
            }
        }
    }
}

impl VRExternalService {
    pub fn new(ptr: VRExternalShmemPtr) -> VRExternalService {
        VRExternalService {
            shmem: ptr.clone(),
            display: None,
        }
    }
}
