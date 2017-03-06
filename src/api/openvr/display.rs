use super::binding as openvr;
use super::binding::ETrackedPropertyError::*;
use super::binding::ETrackedDeviceProperty::*;
use super::binding::EVREye::*;
use super::binding::EVRInitError::*;
use super::binding::ETrackingUniverseOrigin::*;
use super::binding::EGraphicsAPIConvention::*;
use super::library::OpenVRLibrary;
use super::constants;
use super::super::utils;
use std::ffi::CString;
use std::sync::Arc;
use std::cell::RefCell;
use std::slice;
use std::str;
use std::ptr;
use std::mem;
use {VRDisplay, VRDisplayData, VRDisplayCapabilities, VREyeParameters,
    VRFrameData, VRPose, VRStageParameters, VRFieldOfView, VRLayer};

pub type OpenVRDisplayPtr = Arc<RefCell<OpenVRDisplay>>;

pub struct OpenVRDisplay {
    display_id: u64,
    lib: *const OpenVRLibrary,
    index: openvr::TrackedDeviceIndex_t,
    system: *mut openvr::VR_IVRSystem_FnTable,
    chaperone: *mut openvr::VR_IVRChaperone_FnTable,
    compositor: *mut openvr::VR_IVRCompositor_FnTable
}

unsafe impl Send for OpenVRDisplay {}
unsafe impl Sync for OpenVRDisplay {}

impl OpenVRDisplay {
    pub fn new(lib: *const OpenVRLibrary,
               index: openvr::TrackedDeviceIndex_t,
               system: *mut openvr::VR_IVRSystem_FnTable,
               chaperone: *mut openvr::VR_IVRChaperone_FnTable) 
               -> Arc<RefCell<OpenVRDisplay>> {
        Arc::new(RefCell::new(OpenVRDisplay {
            display_id: utils::new_id(),
            lib: lib,
            index: index,
            system: system,
            chaperone: chaperone,
            compositor: ptr::null_mut()
        }))
    }
}

impl Drop for OpenVRDisplay {
     fn drop(&mut self) {
         self.stop_present();
     }
}

impl VRDisplay for OpenVRDisplay {

    fn id(&self) -> u64 {
        self.display_id
    }

    // Returns the current display data.
    fn data(&self) -> VRDisplayData {
        let mut data = VRDisplayData::default();
        
        OpenVRDisplay::fetch_capabilities(&mut data.capabilities);
        self.fetch_eye_parameters(&mut data.left_eye_parameters, &mut data.right_eye_parameters);
        self.fetch_stage_parameters(&mut data);
        data.display_id = self.display_id;
        data.display_name = format!("{} {}",
                            self.get_string_property(ETrackedDeviceProperty_Prop_ManufacturerName_String),
                            self.get_string_property(ETrackedDeviceProperty_Prop_ModelNumber_String));
        data.connected = self.is_connected();

        data
    }

    fn inmediate_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData {
        let mut data = VRFrameData::default();

        let mut tracked_poses: [openvr::TrackedDevicePose_t; openvr::k_unMaxTrackedDeviceCount as usize]
                              = unsafe { mem::uninitialized() };
        unsafe {
            // Calculates updated poses for all displays
            (*self.system).GetDeviceToAbsoluteTrackingPose.unwrap()(ETrackingUniverseOrigin_TrackingUniverseSeated,
                                                                    self.get_seconds_to_photons(),
                                                                    &mut tracked_poses[0],
                                                                    openvr::k_unMaxTrackedDeviceCount);
        };

        let display_pose = &tracked_poses[self.index as usize];
        self.fetch_frame_data(near_z as f32, far_z as f32, &display_pose, &mut data);

        data
    }

     fn synced_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData {
         if self.compositor == ptr::null_mut() {
             // Fallback to inmediate mode if compositor not available
             self.inmediate_frame_data(near_z, far_z);
         }

         let mut display_pose: openvr::TrackedDevicePose_t = unsafe { mem::uninitialized() };
         unsafe {
             (*self.compositor).GetLastPoseForTrackedDeviceIndex.unwrap()(self.index,
                                                                          &mut display_pose,
                                                                          ptr::null_mut());
         }
         let mut data = VRFrameData::default();
         self.fetch_frame_data(near_z as f32, far_z as f32, &display_pose, &mut data);

         data
      }

    // Resets the pose for this display
    fn reset_pose(&mut self) {
        unsafe {
            (*self.system).ResetSeatedZeroPose.unwrap()();
        }
    }

    fn sync_poses(&mut self) {
        if !self.ensure_compositor_ready() {
            return;
        }
        unsafe {
            (*self.compositor).WaitGetPoses.unwrap()(ptr::null_mut(), 0, ptr::null_mut(), 0);
        }
    }

    fn submit_frame(&mut self, layer: &VRLayer) {
        if !self.ensure_compositor_ready() {
            return;
        }
        let mut texture: openvr::Texture_t = unsafe { mem::uninitialized() };
        texture.handle = unsafe { mem::transmute(layer.texture_id as u64) };
        texture.eColorSpace = openvr::EColorSpace::EColorSpace_ColorSpace_Auto;
        texture.eType = EGraphicsAPIConvention_API_OpenGL;

        let mut left_bounds = texture_bounds_to_openvr(&layer.left_bounds);
        let mut right_bounds = texture_bounds_to_openvr(&layer.right_bounds);
        let flags = openvr::EVRSubmitFlags::EVRSubmitFlags_Submit_Default;

        unsafe {
            (*self.compositor).Submit.unwrap()(EVREye_Eye_Left, &mut texture, &mut left_bounds, flags);
            (*self.compositor).Submit.unwrap()(EVREye_Eye_Right, &mut texture, &mut right_bounds, flags);
            (*self.compositor).PostPresentHandoff.unwrap()();
        }
    }

    fn stop_present(&mut self) {
         if self.compositor != ptr::null_mut() {
             println!("ClearLastSubmittedFrame");
             unsafe {
                (*self.compositor).ClearLastSubmittedFrame.unwrap()();
             }
         }
    }
}

impl OpenVRDisplay {
    fn get_string_property(&self, name: openvr::ETrackedDeviceProperty) -> String {
        let max_size = 256;
        let result = String::with_capacity(max_size);
        let mut error = ETrackedPropertyError_TrackedProp_Success;
        let size;
        unsafe {
            size = (*self.system).GetStringTrackedDeviceProperty.unwrap()(self.index, name, 
                                                                          result.as_ptr() as *mut i8, 
                                                                          max_size as u32, 
                                                                          &mut error)
        };

        if size > 0 && error as u32 == ETrackedPropertyError_TrackedProp_Success as u32 {
            let ptr = result.as_ptr() as *mut u8;
            unsafe {
                String::from(str::from_utf8(slice::from_raw_parts(ptr, size as usize)).unwrap_or(""))
            }
        } else {
            "".into()
        }
    }

    fn get_float_property(&self, name: openvr::ETrackedDeviceProperty) -> Option<f32> {
        let mut error = ETrackedPropertyError_TrackedProp_Success;
        let result = unsafe {
            (*self.system).GetFloatTrackedDeviceProperty.unwrap()(self.index, name, &mut error)
        };
        if error as u32 == ETrackedPropertyError_TrackedProp_Success as u32 {
            Some(result)
        } else {
            None
        }
    }

    fn fetch_capabilities(capabilities: &mut VRDisplayCapabilities) {
        capabilities.can_present = true;
        capabilities.has_orientation = true;
        capabilities.has_external_display = true;
        capabilities.has_position = true;
    }

    fn fetch_field_of_view(&self, eye: openvr::EVREye, fov: &mut VRFieldOfView) {
        let (mut up, mut right, mut down, mut left) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        unsafe {
            (*self.system).GetProjectionRaw.unwrap()(eye, &mut left, &mut right, &mut up, &mut down);
        }
        // OpenVR returns clipping plane coordinates in raw tangent units
        // WebVR expects degrees, so we have to convert tangent units to degrees
        fov.up_degrees = -up.atan().to_degrees() as f64;
        fov.right_degrees = right.atan().to_degrees() as f64;
        fov.down_degrees = down.atan().to_degrees() as f64;
        fov.left_degrees = -left.atan().to_degrees() as f64;
    }

    fn is_connected(&self) -> bool {
        unsafe {
            (*self.system).IsTrackedDeviceConnected.unwrap()(self.index)
        }
    }

    fn fetch_eye_parameters(&self, left: &mut VREyeParameters, right: &mut VREyeParameters) {
        self.fetch_field_of_view(EVREye_Eye_Left, &mut left.field_of_view);
        self.fetch_field_of_view(EVREye_Eye_Right, &mut right.field_of_view);

        let (left_matrix, right_matrix) = unsafe {
            ((*self.system).GetEyeToHeadTransform.unwrap()(EVREye_Eye_Left),
             (*self.system).GetEyeToHeadTransform.unwrap()(EVREye_Eye_Right))
        };
        
        left.offset = [left_matrix.m[0][3], left_matrix.m[1][3], left_matrix.m[2][3]];
        right.offset = [right_matrix.m[0][3], right_matrix.m[1][3], right_matrix.m[2][3]];

        let (mut width, mut height) = (0, 0);
        unsafe {
            (*self.system).GetRecommendedRenderTargetSize.unwrap()(&mut width, &mut height);
        }
        left.render_width = width;
        left.render_height = height;
        right.render_width = width;
        right.render_height = height;
    }

    fn fetch_stage_parameters(&self, data: &mut VRDisplayData) {
        // Play area size
        let mut size_x = 0f32;
        let mut size_z = 0f32;

        unsafe {
            (*self.chaperone).GetPlayAreaSize.unwrap()(&mut size_x, &mut size_z);
        }

        if size_x > 0.0 && size_z > 0.0 {
            let matrix: openvr::HmdMatrix34_t = unsafe {
                (*self.system).GetSeatedZeroPoseToStandingAbsoluteTrackingPose.unwrap()()
            };

            data.stage_parameters = Some(VRStageParameters {
                sitting_to_standing_transform: openvr_matrix34_to_array(&matrix),
                size_x: size_x,
                size_z: size_z
            });
        } else {
            
            // Chaperone data not ready yet. HMD might be deactivated.
            // Use some default average transform until data is ready.
            let matrix = [1.0, 0.0, 0.0, 0.0,
                          0.0, 1.0, 0.0, 0.0,
                          0.0, 0.0, 1.0, 0.0,
                          0.0, 0.75, 0.0, 1.0];

            data.stage_parameters = Some(VRStageParameters {
                sitting_to_standing_transform: matrix,
                size_x: 2.0,
                size_z: 2.0
            });
        }
    }

    fn fetch_frame_data(&self,
                        near_z: f32,
                        far_z: f32,
                        display_pose: &openvr::TrackedDevicePose_t,
                        out: &mut VRFrameData) {
        let near_z = near_z as f32;
        let far_z = far_z as f32;
        OpenVRDisplay::fetch_pose(&display_pose, &mut out.pose);
        self.fetch_projection_matrix(EVREye_Eye_Left, near_z, far_z, &mut out.left_projection_matrix);
        self.fetch_projection_matrix(EVREye_Eye_Right, near_z, far_z, &mut out.right_projection_matrix);

        let mut view_matrix: [f32; 16] = unsafe { mem::uninitialized() };
        self.fetch_view_matrix(&display_pose, &mut view_matrix);

        let mut left_eye:[f32; 16] = unsafe { mem::uninitialized() };
        let mut right_eye:[f32; 16] = unsafe { mem::uninitialized() };
        
        // Fech the transform of each eye
        self.fetch_eye_to_head_matrix(EVREye_Eye_Left, &mut left_eye);
        self.fetch_eye_to_head_matrix(EVREye_Eye_Right, &mut right_eye);

        // View matrix must by multiplied by each eye_to_head transformation matrix
        utils::multiply_matrix(&view_matrix, &left_eye, &mut out.left_view_matrix);
        utils::multiply_matrix(&view_matrix, &right_eye, &mut out.right_view_matrix);
        // Invert matrices
        utils::inverse_matrix(&out.left_view_matrix, &mut view_matrix);
        out.left_view_matrix = view_matrix;
        utils::inverse_matrix(&out.right_view_matrix, &mut view_matrix);
        out.right_view_matrix = view_matrix;

        out.timestamp = utils::timestamp();
    }

    fn fetch_projection_matrix(&self, eye: openvr::EVREye, near: f32, far: f32, out: &mut [f32; 16]) {
        let matrix = unsafe {
            (*self.system).GetProjectionMatrix.unwrap()(eye, near, far, EGraphicsAPIConvention_API_OpenGL)
        };
        *out = openvr_matrix44_to_array(&matrix);
    }

    fn fetch_eye_to_head_matrix(&self, eye: openvr::EVREye, out: &mut [f32; 16]) {
        let matrix = unsafe {
            (*self.system).GetEyeToHeadTransform.unwrap()(eye)
        };
        *out = openvr_matrix34_to_array(&matrix);
    }

    pub fn fetch_pose(display_pose:&openvr::TrackedDevicePose_t, out:&mut VRPose) {
        if !display_pose.bPoseIsValid {
            // For some reason the pose may not be valid, return a empty one
            return;
        }

        // OpenVR returns a transformation matrix
        // WebVR expects a quaternion, we have to decompose the transformation matrix
        out.orientation = Some(openvr_matrix_to_quat(&display_pose.mDeviceToAbsoluteTracking));

        // Decompose position from transformation matrix
        out.position = Some(openvr_matrix_to_position(&display_pose.mDeviceToAbsoluteTracking));

        // Copy linear velocity and angular velocity
        out.linear_velocity = Some([display_pose.vVelocity.v[0], 
                                     display_pose.vVelocity.v[1], 
                                     display_pose.vVelocity.v[2]]);
        out.angular_velocity = Some([display_pose.vAngularVelocity.v[0], 
                                      display_pose.vAngularVelocity.v[1], 
                                      display_pose.vAngularVelocity.v[2]]);

        // TODO: OpenVR doesn't expose linear and angular acceleration
        // Derive them from GetDeviceToAbsoluteTrackingPose with different predicted seconds_photons?
    }

    fn fetch_view_matrix(&self, display_pose: &openvr::TrackedDevicePose_t, out: &mut [f32; 16]) {
        if !display_pose.bPoseIsValid {
            *out = identity_matrix!();
        } else {
            *out = openvr_matrix34_to_array(&display_pose.mDeviceToAbsoluteTracking);
        }
    }

    pub fn index(&self) -> openvr::TrackedDeviceIndex_t {
        self.index
    }

    // Computing seconds to photons
    // More info: https://github.com/ValveSoftware/openvr/wiki/IVRSystem::GetDeviceToAbsoluteTrackingPose
    fn get_seconds_to_photons(&self) -> f32 {
        let mut seconds_last_vsync = 0f32;
        let average_value = 0.04f32;

        unsafe {
            if !(*self.system).GetTimeSinceLastVsync.unwrap()(&mut seconds_last_vsync, ptr::null_mut()) {
                // no vsync times are available, return a default average value
                return average_value;
            }
        }
        let display_freq = self.get_float_property(ETrackedDeviceProperty_Prop_DisplayFrequency_Float).unwrap_or(90.0);
        let frame_duration = 1.0 / display_freq;
        if let Some(vsync_to_photons) = self.get_float_property(ETrackedDeviceProperty_Prop_SecondsFromVsyncToPhotons_Float) {
            frame_duration - seconds_last_vsync + vsync_to_photons
        } else {
            0.04f32
        }
    }

    fn ensure_compositor_ready(&mut self)-> bool {
        if self.compositor != ptr::null_mut() {
            return true;
        }

        unsafe {
            let mut error = EVRInitError_VRInitError_None;
            let name = CString::new(format!("FnTable:{}", constants::IVRCompositor_Version)).unwrap();
            self.compositor = (*(*self.lib).get_interface)(name.as_ptr(), &mut error)
                          as *mut openvr::VR_IVRCompositor_FnTable;
            if error as u32 == EVRInitError_VRInitError_None as u32 && self.compositor != ptr::null_mut() {
                // Set seated tracking space (default in WebVR)
                (*self.compositor).SetTrackingSpace.unwrap()(ETrackingUniverseOrigin_TrackingUniverseSeated);
                true
            } else {
                error!("Error initializing OpenVR compositor: {:?}", error as u32);
                self.compositor = ptr::null_mut();
                false
            }
        }
    }
}

// Helper functions
 
#[inline]
fn openvr_matrix34_to_array(matrix: &openvr::HmdMatrix34_t) -> [f32; 16] {
    [matrix.m[0][0], matrix.m[1][0], matrix.m[2][0], 0.0,
     matrix.m[0][1], matrix.m[1][1], matrix.m[2][1], 0.0,
     matrix.m[0][2], matrix.m[1][2], matrix.m[2][2], 0.0,
     matrix.m[0][3], matrix.m[1][3], matrix.m[2][3], 1.0]
}

#[inline]
fn openvr_matrix44_to_array(matrix: &openvr::HmdMatrix44_t) -> [f32; 16] {
    [matrix.m[0][0], matrix.m[1][0], matrix.m[2][0], matrix.m[3][0],
     matrix.m[0][1], matrix.m[1][1], matrix.m[2][1], matrix.m[3][1],
     matrix.m[0][2], matrix.m[1][2], matrix.m[2][2], matrix.m[3][2],
     matrix.m[0][3], matrix.m[1][3], matrix.m[2][3], matrix.m[3][3]]
}

#[inline]
fn openvr_matrix_to_position(matrix: &openvr::HmdMatrix34_t) -> [f32; 3] {
    [matrix.m[0][3], matrix.m[1][3], matrix.m[2][3]]
}

// Adapted from http://www.euclideanspace.com/maths/geometry/rotations/conversions/matrixToQuaternion/index.htm
#[inline]
fn openvr_matrix_to_quat(matrix: &openvr::HmdMatrix34_t) -> [f32; 4] {
    let m = matrix.m;
    let w = f32::max(0.0, 1.0 + m[0][0] + m[1][1] + m[2][2]).sqrt() * 0.5;
    let mut x = f32::max(0.0, 1.0 + m[0][0] - m[1][1] - m[2][2]).sqrt() * 0.5;
    let mut y = f32::max(0.0, 1.0 - m[0][0] + m[1][1] - m[2][2]).sqrt() * 0.5;
    let mut z = f32::max(0.0, 1.0 - m[0][0] - m[1][1] + m[2][2]).sqrt() * 0.5;

    x = copysign(x, m[2][1] - m[1][2]);
    y = copysign(y, m[0][2] - m[2][0]);
    z = copysign(z, m[1][0] - m[0][1]);

    [x, y, z, w]
}

#[inline]
fn copysign(a: f32, b: f32) -> f32 {
    if b == 0.0 {
        0.0
    } else {
        a.abs() * b.signum()
    }
}

fn texture_bounds_to_openvr(bounds: &[f32; 4]) -> openvr::VRTextureBounds_t {
    let mut result: openvr::VRTextureBounds_t = unsafe { mem::uninitialized() };
    // WebVR uses uMin, vMin, uWidth and vHeight bounds
    result.uMin = bounds[0];
    result.vMin = bounds[1];
    result.uMax = result.uMin + bounds[2];
    result.vMax = result.vMin + bounds[3]; 
    result
}
