use std::sync::Arc;
use std::cell::RefCell;
use VRPose;

pub type VRGamepadPtr = Arc<RefCell<VRGamepad>>;

pub trait VRGamepad {
    fn id(&self) -> u32;
    fn data(&self) -> VRGamepadData;
    fn state(&self) -> VRGamepadState;
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadState {
    pub gamepad_id: u32,
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
pub enum VRGamepadHand {
    Unknown,
    Left,
    Right
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadData {
    pub display_id: u32,
    pub name: String,
    pub hand: VRGamepadHand
}

impl Default for VRGamepadData {
     fn default() -> VRGamepadData {
         Self {
            display_id: 0,
            name: String::new(),
            hand: VRGamepadHand::Unknown
         }
     }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRGamepadButton {
    pub pressed: bool,
    pub touched: bool
}

impl VRGamepadButton {
    pub fn new(pressed: bool) -> Self {
        Self {
            pressed: pressed,
            touched: pressed,
        }
    }
}
