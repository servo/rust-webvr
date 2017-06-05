#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRDisplay, VRDisplayData, VRDisplayCapabilities,
    VREvent, VRDisplayEvent, VREyeParameters, VRFrameData, VRLayer};
use ovr_mobile_sys as ovr;
use ovr_mobile_sys::ovrSystemProperty::*;
use std::sync::Arc;
use std::cell::RefCell;
use super::service::OculusVRService;
use super::super::utils;

pub type OculusVRDisplayPtr = Arc<RefCell<OculusVRDisplay>>;

pub struct OculusVRDisplay {
    display_id: u32,
    ovr_java: *const ovr::ovrJava,
}

unsafe impl Send for OculusVRDisplay {}
unsafe impl Sync for OculusVRDisplay {}

impl VRDisplay for OculusVRDisplay {

    fn id(&self) -> u32 {
        self.display_id
    }

    fn data(&self) -> VRDisplayData {
        let mut data = VRDisplayData::default();

        data.display_name = "Oculus VR".into();
        data.display_id = self.display_id;
        data.connected = true;
    
        self.fetch_capabilities(&mut data.capabilities);
        unsafe {
            self.fetch_eye_parameters(&mut data.left_eye_parameters, &mut data.right_eye_parameters);
        }
        data.stage_parameters = None;

        data
    }

    fn inmediate_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let mut data = VRFrameData::default();
        
        data
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let mut data = VRFrameData::default();
        
        data
    }

    fn reset_pose(&mut self) {

    }

    fn sync_poses(&mut self) {

    }

    fn submit_frame(&mut self, layer: &VRLayer) {
        
    }
}

impl OculusVRDisplay {
    pub unsafe fn new(ovr_java: *const ovr::ovrJava) -> Arc<RefCell<OculusVRDisplay>> {
        Arc::new(RefCell::new(OculusVRDisplay {
            display_id: utils::new_id(),
            ovr_java: ovr_java,
        }))
    }

    pub fn pause(&self) {

    }

    pub fn resume(&self) {

    }

    fn fetch_capabilities(&self, capabilities: &mut VRDisplayCapabilities) {
        capabilities.can_present = true;
        capabilities.has_orientation = true;
        capabilities.has_external_display = false;
        capabilities.has_position = false;
    }

    unsafe fn fetch_eye_parameters(&self, left_eye: &mut VREyeParameters, right_eye: &mut VREyeParameters) {
        let fov_x = unsafe {
            ovr::vrapi_GetSystemPropertyFloat(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_FOV_DEGREES_X)
        };
        let fov_y = unsafe {
            ovr::vrapi_GetSystemPropertyFloat(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_FOV_DEGREES_Y)
        };

        left_eye.field_of_view.left_degrees = fov_x as f64;
        left_eye.field_of_view.right_degrees = fov_x as f64;
        left_eye.field_of_view.up_degrees = fov_y as f64;
        left_eye.field_of_view.down_degrees = fov_y as f64;

        right_eye.field_of_view.left_degrees = fov_x as f64;
        right_eye.field_of_view.right_degrees = fov_x as f64;
        right_eye.field_of_view.up_degrees = fov_y as f64;
        right_eye.field_of_view.down_degrees = fov_y as f64;

        let render_size = self.recommended_render_size();
        
        left_eye.render_width = render_size.0;
        left_eye.render_height = render_size.1;
        right_eye.render_width = render_size.0;
        right_eye.render_height = render_size.1;
    }

    fn recommended_render_size(&self) -> (u32, u32) {
        let w = unsafe {
            ovr::vrapi_GetSystemPropertyInt(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_WIDTH)
        };
        let h = unsafe {
            ovr::vrapi_GetSystemPropertyInt(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_TEXTURE_HEIGHT)
        };

        (w as u32, h as u32)
    }

    unsafe fn fetch_frame_data(&self,
                               out: &mut VRFrameData,
                               near: f32,
                               far: f32) {
        let fov_x = unsafe {
            ovr::vrapi_GetSystemPropertyFloat(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_FOV_DEGREES_X)
        };
        let fov_y = unsafe {
            ovr::vrapi_GetSystemPropertyFloat(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_FOV_DEGREES_Y)
        };
        let projection = ovr::helpers::ovrMatrix4f_CreateProjectionFov(fov_x, fov_y, 0.0, 0.0, near, far);
        let projection = ovr_mat4_to_array(&projection);

        out.left_projection_matrix = projection;
        out.right_projection_matrix = projection;

        // Timestamp
        out.timestamp = utils::timestamp();
    }
}

#[inline]
fn ovr_mat4_to_array(matrix: &ovr::ovrMatrix4f) -> [f32; 16] {
    [matrix.M[0][0], matrix.M[1][0], matrix.M[2][0], matrix.M[3][0],
     matrix.M[0][1], matrix.M[1][1], matrix.M[2][1], matrix.M[3][1],
     matrix.M[0][2], matrix.M[1][2], matrix.M[2][2], matrix.M[3][2],
     matrix.M[0][3], matrix.M[1][3], matrix.M[2][3], matrix.M[3][3]]
}
