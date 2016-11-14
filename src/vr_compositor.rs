use VRLayer;

pub trait VRCompositor {
    // Synchronization point to keep in step with the HMD
    // Must be called in the render thread, before doing any work
    fn sync_poses(&mut self);

    // Submits frame to the display
    // Must be called in the render thread
    fn submit_frame(&mut self, layer: &VRLayer);
}