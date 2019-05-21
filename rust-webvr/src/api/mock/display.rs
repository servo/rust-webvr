use {VRDisplay, VRDisplayData, VRFramebuffer, VRFramebufferAttributes, VRFrameData, VRGamepadPtr, VRStageParameters, VRLayer, VRViewport};
use rust_webvr_api::utils;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
pub type MockVRDisplayPtr = Arc<RefCell<MockVRDisplay>>;
use std::time::Duration;
use std::thread;
use super::MockVRControlMsg;

pub struct MockVRDisplay {
    display_id: u32,
    attributes: VRFramebufferAttributes,
    state: Arc<Mutex<MockVRState>>,
}

#[derive(Default)]
pub struct MockVRState {}

unsafe impl Send for MockVRDisplay {}
unsafe impl Sync for MockVRDisplay {}

impl MockVRDisplay {
    pub fn new() -> MockVRDisplayPtr {
        Arc::new(RefCell::new(MockVRDisplay {
            display_id: utils::new_id(),
            attributes: Default::default(),
            state: Default::default(),
        }))
    }

    pub fn state_handle(&self) -> Arc<Mutex<MockVRState>> {
        self.state.clone()
    }
}

impl VRDisplay for MockVRDisplay {

    fn id(&self) -> u32 {
        self.display_id
    }

    fn data(&self) -> VRDisplayData {
        let mut data = VRDisplayData::default();
        
        // Mock display data
        // Simulates a virtual HTC Vive

        data.display_name = "Mock VRDisplay".into();
        data.display_id = self.display_id;
        data.connected = true;

        data.capabilities.can_present = true;
        data.capabilities.has_orientation = true;
        data.capabilities.has_external_display = true;
        data.capabilities.has_position = true;

        data.stage_parameters = Some(VRStageParameters {
            sitting_to_standing_transform: [-0.9317312, 0.0, 0.36314875, 0.0, 0.0, 0.99999994, 0.0, 0.0, -0.36314875, 
                                            0.0, -0.9317312, 0.0, 0.23767996, 1.6813644, 0.45370483, 1.0],
            size_x: 2.0,
            size_z: 2.0
        });

        data.left_eye_parameters.offset = [0.035949998, 0.0, 0.015];
        data.left_eye_parameters.render_width = 1512;
        data.left_eye_parameters.render_height = 1680;
        data.left_eye_parameters.field_of_view.up_degrees = 55.82093048095703;
        data.left_eye_parameters.field_of_view.right_degrees = 51.26948547363281;
        data.left_eye_parameters.field_of_view.down_degrees = 55.707801818847656;
        data.left_eye_parameters.field_of_view.left_degrees = 54.42263412475586;

        data.right_eye_parameters.offset = [-0.035949998, 0.0, 0.015];
        data.right_eye_parameters.render_width = 1512;
        data.right_eye_parameters.render_height = 1680;
        data.right_eye_parameters.field_of_view.up_degrees = 55.898048400878906;
        data.right_eye_parameters.field_of_view.right_degrees = 54.37410354614258;
        data.right_eye_parameters.field_of_view.down_degrees = 55.614715576171875;
        data.right_eye_parameters.field_of_view.left_degrees = 51.304901123046875;
        
        data
    }

    fn immediate_frame_data(&self, _near_z: f64, _far_z: f64) -> VRFrameData {
        let mut data = VRFrameData::default();
        // Position vector
        data.pose.position = Some([0.5, -0.7, -0.3]);
        // Orientation quaternion
        // TODO: Add animation
        data.pose.orientation = Some([0.9385081, -0.08066622, -0.3347714, 0.024972256]);

        // Simulates HTC Vive projections
        data.left_projection_matrix = [0.75620246, 0.0, 0.0, 0.0,
                                       0.0, 0.68050665, 0.0, 0.0,
                                      -0.05713458, -0.0021225351, -1.0000999, -1.0, 
                                       0.0, 0.0, -0.10000999, 0.0];

        data.left_view_matrix = [1.0, 0.0, 0.0, 0.0, 
                                 0.0, 1.0, 0.0, 0.0, 
                                 0.0, 0.0, 1.0, 0.0, 
                                -0.035949998, 0.0, 0.015, 1.0];

        data.right_projection_matrix = [0.75646526, 0.0, 0.0, 0.0, 
                                        0.0, 0.68069947, 0.0, 0.0, 
                                        0.055611316, -0.005315368, -1.0000999, -1.0, 
                                        0.0, 0.0, -0.10000999, 0.0];

        data.right_view_matrix = [1.0, 0.0, 0.0, 0.0,
                                  0.0, 1.0, 0.0, 0.0,
                                  0.0, 0.0, 1.0, 0.0,
                                  0.035949998, 0.0, 0.015, 1.0];

        data.timestamp = utils::timestamp();

        data
    }

    fn synced_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData {
        self.immediate_frame_data(near_z, far_z)
    }

    fn reset_pose(&mut self) {
        // No op
    }

    fn sync_poses(&mut self) {
        // Simulate Vsync
        thread::sleep(Duration::from_millis(1));
    }

    fn bind_framebuffer(&mut self, _index: u32) {
        // No op
    }

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        vec![VRFramebuffer {
                eye_index: 0,
                attributes: self.attributes,
                viewport: VRViewport::new(0, 0, 1512/2, 1680)
            },
            VRFramebuffer {
                eye_index: 1,
                attributes: self.attributes,
                viewport: VRViewport::new(1512/2, 0, 1512/2, 1680)
            }]
    }

    fn render_layer(&mut self, _layer: &VRLayer) {
        // No op
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        Ok(Vec::new())
    }

    fn submit_frame(&mut self) {
        // No op
    }

    fn start_present(&mut self, attributes: Option<VRFramebufferAttributes>) {
        if let Some(attributes) = attributes {
            self.attributes = attributes;
        }
    }
}

impl MockVRState {
    pub fn handle_msg(&mut self, msg: MockVRControlMsg) {
        match msg {

        }
    }
}