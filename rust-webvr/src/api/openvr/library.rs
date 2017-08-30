use libloading as lib;
#[cfg(unix)]
use libloading::os::unix::Symbol as Symbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as Symbol;

use super::binding as openvr;

// OpenVR_api.dll entry points
type VRInitInternal = unsafe extern fn(*mut openvr::EVRInitError, openvr::EVRApplicationType) -> isize;
type VRShutdownInternal = unsafe extern fn();
type VRIsHmdPresent = unsafe extern fn() -> bool;
type VRGetGenericInterface = unsafe extern fn(*const ::std::os::raw::c_char, *mut openvr::EVRInitError) -> isize;

pub struct OpenVRLibrary {
    _lib: lib::Library,
    pub init_internal: Symbol<VRInitInternal>,
    pub shutdown_internal: Symbol<VRShutdownInternal>,
    pub is_hmd_present: Symbol<VRIsHmdPresent>,
    pub get_interface: Symbol<VRGetGenericInterface>
}

impl OpenVRLibrary {
    pub unsafe fn new()-> lib::Result<OpenVRLibrary> {
        let lib = try!(lib::Library::new("openvr_api.dll"));
        let init_internal = try!(lib.get::<VRInitInternal>(b"VR_InitInternal\0")).into_raw();
        let shutdown_internal = try!(lib.get::<VRShutdownInternal>(b"VR_ShutdownInternal\0")).into_raw();
        let is_hmd_present = try!(lib.get::<VRIsHmdPresent>(b"VR_IsHmdPresent\0")).into_raw();
        let get_interface = try!(lib.get::<VRGetGenericInterface>(b"VR_GetGenericInterface\0")).into_raw();
        
        Ok(OpenVRLibrary {
            _lib: lib,
            init_internal: init_internal,
            shutdown_internal: shutdown_internal,
            is_hmd_present: is_hmd_present,
            get_interface: get_interface
        })
    }
}