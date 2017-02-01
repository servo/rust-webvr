use std::sync::Arc;
use std::cell::RefCell;
use VRPose;

pub type VRGamepadPtr = Arc<RefCell<VRGamepad>>;

pub trait VRGamepad {
    fn id(&self) -> u64;
    fn name(&self) -> String;
    fn state(&self) -> VRGamepadState;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadState {
    pub connected: bool,
    pub timestamp: f64,
    pub axes: Vec<f64>,
    pub buttons: Vec<VRGamepadButton>,
    pub pose: VRPose
}

impl Default for VRGamepadState {
     fn default() -> VRGamepadState {
         VRGamepadState {
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
pub struct VRGamepadButton {
    pub pressed: bool,
    pub touched: bool
}