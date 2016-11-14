use VRCompositor;
use VRLayer;

pub struct MockVRCompositor;

impl MockVRCompositor {
    pub fn new() -> MockVRCompositor {
        MockVRCompositor {
        }
    }
}

impl VRCompositor for MockVRCompositor {
    fn sync_poses(&mut self) {

    }

    fn submit_frame(&mut self, _layer: &VRLayer) {

    }
}

