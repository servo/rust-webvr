use std::sync::Arc;
use std::cell::RefCell;
use VRPose;

pub type VRGamepadPtr = Arc<RefCell<VRGamepad>>;

pub trait VRGamepad {
    fn id(&self) -> u64;
    fn data(&self) -> VRGamepadData;
    fn state(&self) -> VRGamepadState;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadState {
    pub gamepad_id: u64,
    pub connected: bool,
    pub timestamp: f64,
    pub axes: Vec<f64>,
    pub buttons: Vec<VRGamepadButton>,
    pub pose: VRPose
}

impl Default for VRGamepadState {
     fn default() -> VRGamepadState {
         VRGamepadState {
            gamepad_id: 0,
            connected: false,
            timestamp: 0.0,
            axes: Vec::new(),
            buttons: Vec::new(),
            pose: VRPose::default()
         }
     }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadData {
    pub display_id: u64,
    pub name: String
}

impl Default for VRGamepadData {
     fn default() -> VRGamepadData {
         Self {
            display_id: 0,
            name: String::new()
         }
     }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadButton {
    pub pressed: bool,
    pub touched: bool
}
