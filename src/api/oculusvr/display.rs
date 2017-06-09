#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRDisplay, VRDisplayData, VRDisplayCapabilities,
    VREvent, VRDisplayEvent, VREyeParameters, VRFrameData, VRLayer};
use android_injected_glue;
use gl;
use ovr_mobile_sys as ovr;
use ovr_mobile_sys::ovrFrameLayerEye::*;
use ovr_mobile_sys::ovrSystemProperty::*;
use std::sync::Arc;
use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr;
use super::service::OculusVRService;
use super::super::utils;

pub type OculusVRDisplayPtr = Arc<RefCell<OculusVRDisplay>>;

extern {
    fn eglGetCurrentContext() -> *mut ::std::os::raw::c_void;
    fn eglGetCurrentDisplay() -> *mut ::std::os::raw::c_void;
}

pub struct OculusVRDisplay {
    display_id: u32,
    ovr: *mut ovr::ovrMobile,
    ovr_java: *const ovr::ovrJava,
    eye_framebuffers: Vec<OculusEyeFramebuffer>,
    read_fbo: u32,
    read_texture: u32,
    resumed: bool,
    frame_index: i64,
    predicted_display_time: f64,
    predicted_tracking: ovr::ovrTracking,
    eye_projection: Cell<ovr::ovrMatrix4f>,
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
        self.fetch_eye_parameters(&mut data.left_eye_parameters, &mut data.right_eye_parameters);
        
        data.stage_parameters = None;

        data
    }

    fn inmediate_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let mut data = VRFrameData::default();

        if self.is_in_vr_mode() {
            let tracking = unsafe { ovr::vrapi_GetPredictedTracking(self.ovr, 0.0) };
            self.fetch_frame_data(&tracking, &mut data, near as f32, far as f32);
        }
        
        data
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let mut data = VRFrameData::default();
        if self.is_in_vr_mode() {
            self.fetch_frame_data(&self.predicted_tracking, &mut data, near as f32, far as f32);
        }

        data
    }

    fn reset_pose(&mut self) {
        if self.is_in_vr_mode() {
            unsafe {
                ovr::vrapi_RecenterPose(self.ovr);
            }
        }
    }

    fn sync_poses(&mut self) {
        if !self.is_in_vr_mode() {
            self.start_present();
        }
        if self.eye_framebuffers.is_empty() {
            self.create_swap_chains();
            debug_assert!(!self.eye_framebuffers.is_empty());
        }
        self.frame_index += 1;
        self.predicted_display_time =  unsafe { ovr::vrapi_GetPredictedDisplayTime(self.ovr, self.frame_index) };
        self.predicted_tracking = unsafe { ovr::vrapi_GetPredictedTracking(self.ovr, self.predicted_display_time) };
    }

    fn submit_frame(&mut self, layer: &VRLayer) {
        let mut frame_params = ovr::helpers::vrapi_DefaultFrameParms(self.ovr_java,
                                                                     ovr::ovrFrameInit::VRAPI_FRAME_INIT_DEFAULT,
                                                                     self.predicted_display_time,
                                                                     ptr::null_mut());
        frame_params.FrameIndex = self.frame_index;

        // Save current fbo to restore it when the frame is submitted.
        let mut current_fbo = 0;
        unsafe {
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut current_fbo);
        }

        let eye_projection = self.eye_projection.get();
        for (i, eye) in self.eye_framebuffers.iter_mut().enumerate() {
            let swap_chain_length = unsafe { ovr::vrapi_GetTextureSwapChainLength(eye.swap_chain) };
            let swap_chain_index = (self.frame_index % swap_chain_length as i64) as i32;

            if self.read_texture != layer.texture_id {
                // Attach external texture to the used later in BlitFramebuffer.
                unsafe {
                    gl::BindFramebuffer(gl::FRAMEBUFFER, self.read_fbo);
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER,
                                             gl::COLOR_ATTACHMENT0,
                                             gl::TEXTURE_2D,
                                             layer.texture_id, 0);
                }
                self.read_texture = layer.texture_id;
            }

            let texture_size = layer.texture_size.unwrap_or_else(|| {
                (eye.width * 2, eye.height)
            });

            // BlitFramebuffer: external texture to gvr pixel buffer.
            unsafe {
                gl::BindFramebuffer(gl::FRAMEBUFFER, eye.fbos[swap_chain_index as usize]);
                gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.read_fbo);
                let w = texture_size.0/2;
                let x0 = (i as u32) * w;
                let x1 = x0 + w;
                gl::BlitFramebuffer(x0 as i32, 0, x1 as i32, texture_size.1 as i32,
                                    0, 0, eye.width as i32, eye.height as i32,
                                    gl::COLOR_BUFFER_BIT, gl::LINEAR);
            }

            let target = &mut frame_params.Layers[0].Textures[i];
            target.ColorTextureSwapChain = eye.swap_chain;
            target.TextureSwapChainIndex = swap_chain_index;
            target.TexCoordsFromTanAngles = ovr::helpers::ovrMatrix4f_TanAngleMatrixFromProjection(&eye_projection);
            target.HeadPose = self.predicted_tracking.HeadPose;
            //frame_params.Layers[0].Textures[eye].CompletionFence = fence;
        }

        unsafe {
            ovr::vrapi_SubmitFrame(self.ovr, &frame_params);
            // Restore bound fbo
            gl::BindFramebuffer(gl::FRAMEBUFFER, current_fbo as u32);
        }
    }

    fn start_present(&mut self) {
        if !self.ovr.is_null() {
            return;
        }
        
        let mut mode = ovr::helpers::vrapi_DefaultModeParms(self.ovr_java);
        mode.Flags |= ovr::ovrModeFlags::VRAPI_MODE_FLAG_NATIVE_WINDOW as u32;
        //mode.WindowSurface = unsafe { android_injected_glue::get_native_window() as u64 };
        //mode.Display = unsafe { eglGetCurrentDisplay() as u64 };
        //mode.ShareContext = unsafe { eglGetCurrentContext() as u64 };

        self.ovr = unsafe { ovr::vrapi_EnterVrMode(&mode) };

        if self.ovr.is_null() {
            error!("Entering VR mode failed because the ANativeWindow was not valid.");
        }
    }

    fn stop_present(&mut self) {
        if !self.ovr.is_null() {
            return;
        }
        unsafe {
            ovr::vrapi_LeaveVrMode(self.ovr);
        }
        self.ovr = ptr::null_mut();
    }
}

impl OculusVRDisplay {
    pub fn new(ovr_java: *const ovr::ovrJava) -> Arc<RefCell<OculusVRDisplay>> {
        Arc::new(RefCell::new(OculusVRDisplay {
            display_id: utils::new_id(),
            ovr: ptr::null_mut(),
            ovr_java: ovr_java,
            eye_framebuffers: Vec::new(),
            read_fbo: 0,
            read_texture: 0,
            resumed: true,
            frame_index: 0,
            predicted_display_time: 0.0,
            predicted_tracking: unsafe { mem::zeroed() },
            eye_projection: Cell::new(ovr::helpers::ovrMatrix4f_CreateIdentity()),
        }))
    }

    pub fn pause(&mut self) {
        self.resumed = false;
    }

    pub fn resume(&mut self) {
        self.resumed = true;
    }

    fn is_in_vr_mode(&self) -> bool {
        self.resumed && !self.ovr.is_null()
    }

    fn create_swap_chains(&mut self) {
        self.eye_framebuffers.clear();

        let recommended_eye_size = self.recommended_render_size();

        if self.read_fbo == 0 {
            let mut fbo = 0;
            unsafe {
                gl::GenFramebuffers(1, &mut fbo);
            }
            self.read_fbo = fbo as u32;
        }

        // Save current state
        let mut current_fbo = 0;
        let mut current_texture = 0;
        unsafe {
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut current_fbo);
            gl::GetIntegerv(gl::TEXTURE_BINDING_2D, &mut current_texture);
        }

        for _ in 0..2 {
            let eye_framebuffer = unsafe {
                OculusEyeFramebuffer::new(recommended_eye_size.0, recommended_eye_size.1)
            };
            self.eye_framebuffers.push(eye_framebuffer);
        }

        // Restore VRGamepadState
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, current_fbo as u32);
            gl::BindFramebuffer(gl::TEXTURE_2D, current_texture as u32);
        }
    }

    fn fetch_capabilities(&self, capabilities: &mut VRDisplayCapabilities) {
        capabilities.can_present = true;
        capabilities.has_orientation = true;
        capabilities.has_external_display = false;
        capabilities.has_position = false;
    }

    fn fetch_eye_parameters(&self, left_eye: &mut VREyeParameters, right_eye: &mut VREyeParameters) {
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

    fn fetch_frame_data(&self,
                        tracking: &ovr::ovrTracking,
                        out: &mut VRFrameData,
                        near: f32,
                        far: f32) {
        let fov_x = unsafe {
            ovr::vrapi_GetSystemPropertyFloat(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_FOV_DEGREES_X)
        };
        let fov_y = unsafe {
            ovr::vrapi_GetSystemPropertyFloat(self.ovr_java, VRAPI_SYS_PROP_SUGGESTED_EYE_FOV_DEGREES_Y)
        };

        // Projection Matrix
        let projection = ovr::helpers::ovrMatrix4f_CreateProjectionFov(fov_x, fov_y, 0.0, 0.0, near, far);
        self.eye_projection.set(projection); // Will be used in submit Frame.
        let projection = ovr_mat4_to_array(&projection);

        out.left_projection_matrix = projection;
        out.right_projection_matrix = projection;

        // View Matrix
        let model_params = ovr::helpers::vrapi_DefaultHeadModelParms();
        let tracking = ovr::helpers::vrapi_ApplyHeadModel(&model_params, tracking);
        
        let center_matrix = ovr::helpers::vrapi_GetCenterEyeViewMatrix(&model_params, &tracking, None);
        let left_eye_view_matrix = ovr::helpers::vrapi_GetEyeViewMatrix(&model_params,
                                                                        &center_matrix,
                                                                        VRAPI_FRAME_LAYER_EYE_LEFT as i32);
        let right_eye_view_matrix = ovr::helpers::vrapi_GetEyeViewMatrix(&model_params,
                                                                         &center_matrix,
                                                                         VRAPI_FRAME_LAYER_EYE_RIGHT as i32);
        out.left_view_matrix = ovr_mat4_to_array(&left_eye_view_matrix);
        out.right_view_matrix = ovr_mat4_to_array(&right_eye_view_matrix);

        // Pose
        out.pose.orientation = Some(ovr_quat_to_array(&tracking.HeadPose.Pose.Orientation));
        out.pose.position = Some(ovr_vec3_to_array(&tracking.HeadPose.Pose.Position));
        out.pose.linear_velocity = Some(ovr_vec3_to_array(&tracking.HeadPose.LinearVelocity));
        out.pose.linear_acceleration = Some(ovr_vec3_to_array(&tracking.HeadPose.LinearAcceleration));
        out.pose.angular_velocity = Some(ovr_vec3_to_array(&tracking.HeadPose.AngularVelocity));
        out.pose.angular_acceleration = Some(ovr_vec3_to_array(&tracking.HeadPose.AngularAcceleration));

        // Timestamp
        out.timestamp = tracking.HeadPose.TimeInSeconds * 1000.0;
    }
}

struct OculusEyeFramebuffer {
    swap_chain: *mut ovr::ovrTextureSwapChain,
    swap_chain_length: i32,
    fbos: Vec<u32>, // Multiple FBOs for triple buffering
    width: u32,
    height: u32
}

impl OculusEyeFramebuffer {
    pub unsafe fn new(width: u32, height: u32) -> OculusEyeFramebuffer {
        let swap_chain = ovr::vrapi_CreateTextureSwapChain(ovr::ovrTextureType::VRAPI_TEXTURE_TYPE_2D,
                                                           ovr::ovrTextureFormat::VRAPI_TEXTURE_FORMAT_8888,
                                                           width as i32,
                                                           height as i32,
                                                           1,
                                                           true);
        let swap_chain_length = ovr::vrapi_GetTextureSwapChainLength(swap_chain);
        let mut fbos = Vec::new();
        for index in 0..swap_chain_length {
            // Initialize the color buffer texture.
            let texture = ovr::vrapi_GetTextureSwapChainHandle(swap_chain, index);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            // Set up the FBO to render to the texture.
            let mut fbo = 0;
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);
            gl::FramebufferTexture2D(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, texture, 0);
            let status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);
            if status != gl::FRAMEBUFFER_COMPLETE {
                panic!("Oculus VR Incomplete Framebuffer: {}", status);
            }
            fbos.push(fbo);
        }

        OculusEyeFramebuffer {
            swap_chain: swap_chain,
            swap_chain_length: swap_chain_length,
            fbos: fbos,
            width: width,
            height: height,
        }
    }
}

impl Drop for OculusEyeFramebuffer {
    fn drop(&mut self) {
        unsafe {
            for fbo in &self.fbos {
                gl::DeleteFramebuffers(1, mem::transmute(fbo));
            }
            ovr::vrapi_DestroyTextureSwapChain(self.swap_chain);
        }
    }
}

#[inline]
fn ovr_mat4_to_array(matrix: &ovr::ovrMatrix4f) -> [f32; 16] {
    [matrix.M[0][0], matrix.M[1][0], matrix.M[2][0], matrix.M[3][0],
     matrix.M[0][1], matrix.M[1][1], matrix.M[2][1], matrix.M[3][1],
     matrix.M[0][2], matrix.M[1][2], matrix.M[2][2], matrix.M[3][2],
     matrix.M[0][3], matrix.M[1][3], matrix.M[2][3], matrix.M[3][3]]
}

#[inline]
fn ovr_quat_to_array(q: &ovr::ovrQuatf) -> [f32; 4] {
    [q.x, q.y, q.z, q.w]
}

#[inline]
fn ovr_vec3_to_array(v: &ovr::ovrVector3f) -> [f32; 3] {
    [v.x, v.y, v.z]
}
