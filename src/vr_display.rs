use {VRDisplayData, VRFrameData, VRLayer};
use std::sync::Arc;
use std::cell::RefCell;
pub type VRDisplayPtr = Arc<RefCell<VRDisplay>>;

// The VRDisplay traits forms the base of all VR device implementations
pub trait VRDisplay: Send + Sync {

    // Returns unique device identifier
    fn id(&self) -> u64;

    // Returns the current display data.
    fn data(&self) -> VRDisplayData;

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

    // Hint to indicate that we are goig to start sending frames to the device
    fn start_present(&mut self) {}

    // Hint to indicate that we are goig to stop sending frames to the device
    fn stop_present(&mut self) {}
}

impl PartialEq for VRDisplay {
    fn eq(&self, other: &VRDisplay) -> bool {
        self.id() == other.id()
    }
}