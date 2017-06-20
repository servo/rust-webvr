#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRGamepad, VRGamepadData, VRGamepadHand, VRGamepadState};
use super::super::utils;
use std::cell::RefCell;
use std::sync::Arc;

pub type OculusVRGamepadPtr = Arc<RefCell<OculusVRGamepad>>;

pub struct OculusVRGamepad {
    gamepad_id: u32,
    display_id: u32,
}

unsafe impl Send for OculusVRGamepad {}
unsafe impl Sync for OculusVRGamepad {}

impl OculusVRGamepad {
    pub unsafe fn new(display_id: u32)
                      -> Result<Arc<RefCell<OculusVRGamepad>>, String> {
        let gamepad = Self {
            gamepad_id: utils::new_id(),
            display_id: display_id,
        };

        Ok(Arc::new(RefCell::new(gamepad)))
    }

    // Warning: this function is called from java Main thread
    // The action it's handled in handle_events method for thread safety
    #[allow(dead_code)]
    pub fn pause(&mut self) {
        
    }

    // Warning: this function is called from java Main thread
    // The action it's handled in handle_events method for thread safety
    #[allow(dead_code)]
    pub fn resume(&mut self) {
        
    }
}

impl Drop for OculusVRGamepad {
    fn drop(&mut self) {
    }
}

impl VRGamepad for OculusVRGamepad {
    fn id(&self) -> u32 {
        self.gamepad_id
    }

    fn data(&self) -> VRGamepadData {
        VRGamepadData {
            display_id: self.display_id,
            name: "OculusVR".into(),
            hand: VRGamepadHand::Right
        }
    }

    fn state(&self) -> VRGamepadState {
        let mut out = VRGamepadState::default();

        out.gamepad_id = self.gamepad_id;

        out
    }
}
