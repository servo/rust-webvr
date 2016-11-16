use {VRDisplayData, VRFrameData, VRLayer};
use std::sync::Arc;
use std::cell::RefCell;
pub type VRDevicePtr = Arc<RefCell<VRDevice>>;

// The VRDevice traits forms the base of all VR device implementations
pub trait VRDevice: Send + Sync {

    // Returns unique device identifier
    fn device_id(&self) -> u64;

    // Returns the current display data.
    fn display_data(&self) -> VRDisplayData;

    // Returns the inmediate VRFrameData of the HMD
    // Shpuld be used when not presenting to the device.
    fn inmediate_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData;

    // Returns the synced VRFrameData to render the current frame.
    // Should be used when presenting to the device.
    // sync_poses must have been called before this call.
    fn synced_frame_data(&self, next: f64, far_z: f64) -> VRFrameData;

    // Resets the pose for this display
    fn reset_pose(&mut self);

    // Synchronization point to keep in step with the HMD
    // Returns VRFrameData to be used in the next render frame
    // Must be called in the render thread, before doing any work
    fn sync_poses(&mut self);

    // Submits frame to the display
    // Must be called in the render thread
    fn submit_frame(&mut self, layer: &VRLayer);
}

impl PartialEq for VRDevice {
    fn eq(&self, other: &VRDevice) -> bool {
        self.device_id() == other.device_id()
    }
}