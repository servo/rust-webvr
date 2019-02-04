use std::os::raw::c_void;

pub struct VRExternalShmemPtr;

impl VRExternalShmemPtr {
    pub fn new(_: *mut c_void) -> VRExternalShmemPtr {
        VRExternalShmemPtr
    }
}
