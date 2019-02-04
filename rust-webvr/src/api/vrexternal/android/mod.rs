mod display;
mod mozgfx;
mod service;

use std::os::raw::c_void;
use {VRService, VRServiceCreator};

#[derive(Clone)]
pub struct VRExternalShmemPtr(*mut mozgfx::VRExternalShmem);

unsafe impl Send for VRExternalShmemPtr {}
unsafe impl Sync for VRExternalShmemPtr {}

impl VRExternalShmemPtr {
    fn as_mut(&self) -> &mut mozgfx::VRExternalShmem {
        unsafe { &mut *(self.0) }
    }

    pub fn new(raw: *mut c_void) -> VRExternalShmemPtr {
        VRExternalShmemPtr(raw as *mut mozgfx::VRExternalShmem)
    }
}

pub struct VRExternalServiceCreator(VRExternalShmemPtr);

impl VRExternalServiceCreator {
    pub fn new(ptr: VRExternalShmemPtr) -> Box<VRServiceCreator> {
        Box::new(VRExternalServiceCreator(ptr))
    }
}

impl VRServiceCreator for VRExternalServiceCreator {
    fn new_service(&self) -> Box<VRService> {
        Box::new(service::VRExternalService::new(self.0.clone()))
    }
}
