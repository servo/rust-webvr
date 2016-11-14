use VRDisplayData;
use VRFrameData;
use VRCompositor;
use std::sync::Arc;
use std::cell::RefCell;
pub type VRDevicePtr = Arc<RefCell<VRDevice>>;

// The VRDevice traits forms the base of all VR device implementations
pub trait VRDevice: Send + Sync {

    // Returns unique device identifier
    fn device_id(&self) -> u64;

    // Returns the device type
    fn device_type(&self) -> VRDeviceType;

    // Returns the current display data.
    fn get_display_data(&self) -> VRDisplayData;

    // Returns the VRFrameData with the information required to render the current frame.
    fn get_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData;

    // Resets the pose for this display
    fn reset_pose(&mut self);

    // creates a compositor which allows to sync and submit frames to the HMD
    fn create_compositor(&self) -> Result<Box<VRCompositor>, String>;
}

impl PartialEq for VRDevice {
    fn eq(&self, other: &VRDevice) -> bool {
        self.device_id() == other.device_id()
    }
}

// Enum of all available Implementations
// This is used to send VRCompositor creation commands across ipc-channels.
#[allow(unused_attributes)]
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum VRDeviceType {
    Mock = 1,
    OpenVR = 2
}

impl VRDeviceType {
    pub fn from_u32(val: u32) -> Option<VRDeviceType> {
        match val {
            1 => Some(VRDeviceType::Mock),
            2 => Some(VRDeviceType::OpenVR),
            _ => None
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }
}
