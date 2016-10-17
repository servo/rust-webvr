use VRDisplayData;
use VRPose;
use VRFrameData;
use VRLayer;
use std::sync::Arc;
use std::cell::RefCell;
pub type VRDevicePtr = Arc<RefCell<VRDevice>>;

// The VRDevice traits forms the base of all VR device implementations
pub trait VRDevice: Send {

    // Returns unique device identifier
    fn device_id(&self) -> u64;

    // Returns the current display data.
    fn get_display_data(&self) -> VRDisplayData;

    // Returns the VRFrameData with the information required to render the current frame.
    fn get_frame_data(&self, near_z: f32, far_z: f32) -> VRFrameData;

    // Resets the pose for this display
    fn reset_pose(&mut self);

    // Synchronization point to keep in step with the HMD
    // Must be called in the render thread, before doing any work
    fn sync_poses(&mut self);

    // Submits frame to the display
    fn submit_frame(&mut self, layer: &VRLayer);
}

impl PartialEq for VRDevice {
    fn eq(&self, other: &VRDevice) -> bool {
        self.device_id() == other.device_id()
    }
}