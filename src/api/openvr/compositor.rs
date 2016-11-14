use super::constants;
use super::openvr_sys as openvr;
use super::openvr_sys::EGraphicsAPIConvention::*;
use super::openvr_sys::EVRInitError::*;
use super::openvr_sys::EVREye::*;
use std::ffi::CString;
use std::mem;
use std::ptr;
use VRLayer;
use VRCompositor;

pub struct OpenVRCompositor {
    compositor: *mut openvr::VR_IVRCompositor_FnTable
}

impl OpenVRCompositor {
     pub fn new() -> Result<OpenVRCompositor, String> {
        unsafe {
            let mut error = EVRInitError_VRInitError_None;
            let name = CString::new(format!("FnTable:{}",constants::IVRCOMPOSITOR_VERSION)).unwrap();
            let compositor = openvr::VR_GetGenericInterface(name.as_ptr(), &mut error)
                          as *mut openvr::VR_IVRCompositor_FnTable;
            if error as u32 == EVRInitError_VRInitError_None as u32 && compositor != ptr::null_mut() {
                Ok(OpenVRCompositor {
                    compositor: compositor
                })
            } else {
                Err(format!("Error initializing OpenVR compositor: {:?}", error as u32))
            }
        }
     }
}

impl VRCompositor for OpenVRCompositor {

    fn sync_poses(&mut self) {
        unsafe {
            let mut tracked_poses: [openvr::TrackedDevicePose_t; constants::K_UNMAXTRACKEDDEVICECOUNT as usize]
                              = mem::uninitialized();
            (*self.compositor).WaitGetPoses.unwrap()(&mut tracked_poses[0], 
                                                 constants::K_UNMAXTRACKEDDEVICECOUNT, 
                                                 ptr::null_mut(), 0);
        }
    }

    fn submit_frame(&mut self, layer: &VRLayer) {
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
