use super::binding as openvr;
use super::binding::EVRInitError::*;
use super::binding::EVRApplicationType::*;
use super::binding::ETrackedDeviceClass::*;
use super::binding::EVREventType::*;
use super::constants;
use super::display::{OpenVRDisplay, OpenVRDisplayPtr};
use super::gamepad::{OpenVRGamepad, OpenVRGamepadPtr};
use super::library::OpenVRLibrary;
use std::ffi::CString;
use std::ptr;
use std::mem;
use {VRService, VRDisplay, VRDisplayPtr, VRDisplayEvent, VRDisplayEventReason, VRGamepadPtr};

// OpenVR Service implementation
pub struct OpenVRService {
    initialized: bool,
    lib: Option<OpenVRLibrary>,
    displays: Vec<OpenVRDisplayPtr>,
    gamepads: Vec<OpenVRGamepadPtr>,
    system: *mut openvr::VR_IVRSystem_FnTable,
    chaperone: *mut openvr::VR_IVRChaperone_FnTable,
}

unsafe impl Send for OpenVRService {}

impl VRService for OpenVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        if self.initialized {
            return Ok(());
        }

        // Load OpenVR library
        match unsafe { OpenVRLibrary::new() } {
            Ok(lib) => self.lib = Some(lib),
            Err(msg) => {
                return Err(format!("Error loading OpenVR dll: {:?}", msg));
            }
        };

        if !self.is_available() {
            return Err("Not available".into());
        }

        // Initialize OpenVR
        let mut error = EVRInitError_VRInitError_None;
        unsafe {
             (*self.lib.as_ref().unwrap().init_internal)(&mut error, EVRApplicationType_VRApplication_Scene);
        }

        if error as u32 != EVRInitError_VRInitError_None as u32 {
            return Err(format!("OpenVR Internal failed with error {}", error as u32));
        }

        // Initialize System
        error = EVRInitError_VRInitError_None;
        unsafe {
            let name = CString::new(format!("FnTable:{}", constants::IVRSystem_Version)).unwrap();
            self.system = (*self.lib.as_ref().unwrap().get_interface)(name.as_ptr(), &mut error)
                          as *mut openvr::VR_IVRSystem_FnTable;
        }

        if error as u32 != EVRInitError_VRInitError_None as u32 {
            return Err(format!("OpenVR GetGenericInterface failed with error {}", error as u32));
        }

        // Initialize Chaperone
        error = EVRInitError_VRInitError_None;
        unsafe {
            let name = CString::new(format!("FnTable:{}", constants::IVRChaperone_Version)).unwrap();
            self.chaperone = (*self.lib.as_ref().unwrap().get_interface)(name.as_ptr(), &mut error)
                             as *mut openvr::VR_IVRChaperone_FnTable;
        }
          
        if error as u32 != EVRInitError_VRInitError_None as u32 {
            return Err(format!("OpenVR GetGenericInterface failed with error {:?}", error));
        }

        self.initialized = true;
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        // Return cached displays if available
        if self.initialized && self.displays.len() > 0 {
            return Ok(self.clone_displays());
        }
        // Ensure that there are not initialization errors
        try!(self.initialize());

        let max_device_count: u32 = openvr::k_unMaxTrackedDeviceCount;
        self.displays.clear();

        for i in 0..max_device_count {
            let device_class: openvr::ETrackedDeviceClass = unsafe {
                (*self.system).GetTrackedDeviceClass.unwrap()(i as openvr::TrackedDeviceIndex_t)
            };
            
            match device_class {
                ETrackedDeviceClass_TrackedDeviceClass_HMD => {
                    self.displays.push(OpenVRDisplay::new(self.lib.as_ref().unwrap(), i, self.system, self.chaperone));
                },
                _ => {}
            }
        }


        Ok(self.clone_displays())
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        // Return cached gamepads if available
        if self.initialized && self.gamepads.len() > 0 {
            return Ok(self.clone_gamepads());
        }
        try!(self.initialize());

        let max_device_count: u32 = openvr::k_unMaxTrackedDeviceCount;
        self.gamepads.clear();

        for i in 0..max_device_count {
            let device_class: openvr::ETrackedDeviceClass = unsafe {
                (*self.system).GetTrackedDeviceClass.unwrap()(i as openvr::TrackedDeviceIndex_t)
            };

            match device_class {
                ETrackedDeviceClass_TrackedDeviceClass_Controller => {
                    self.gamepads.push(OpenVRGamepad::new(i, self.system));
                },
                _ => {}
            }
        }

        Ok(self.clone_gamepads())
    }

    fn is_available(&self) -> bool {
        unsafe {
            match self.lib {
                Some(ref lib) => (*lib.is_hmd_present)(),
                None => false
            }
        }
    }

    fn poll_events(&self) -> Vec<VRDisplayEvent> {
        let mut result = Vec::new();
        if !self.initialized || self.system.is_null() {
            return result;
        }
        let mut event: openvr::VREvent_t = unsafe { mem::uninitialized() };
        let size = mem::size_of::<openvr::VREvent_t>() as u32;
        while unsafe { (*self.system).PollNextEvent.unwrap()(&mut event, size) } {

            let event_type: openvr::EVREventType = unsafe { mem::transmute(event.eventType) };

            match event_type {
                EVREventType_VREvent_TrackedDeviceUserInteractionStarted => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
    
                        result.push(VRDisplayEvent::Activate(display.borrow().data(), 
                                                                   VRDisplayEventReason::Mounted));
                    }
                },
                EVREventType_VREvent_TrackedDeviceUserInteractionEnded => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
    
                        result.push(VRDisplayEvent::Deactivate(display.borrow().data(), 
                                                                     VRDisplayEventReason::Unmounted));
                    }
                },
                EVREventType_VREvent_TrackedDeviceActivated => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Connect(display.borrow().data()))
                    }
                },
                EVREventType_VREvent_TrackedDeviceDeactivated => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Disconnect(display.borrow().id()))
                    }
                },
                EVREventType_VREvent_DashboardActivated => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Blur(display.borrow().data()))
                    }
                },
                EVREventType_VREvent_DashboardDeactivated => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Focus(display.borrow().data()))
                    }
                },
                EVREventType_VREvent_ChaperoneDataHasChanged |
                EVREventType_VREvent_IpdChanged |
                EVREventType_VREvent_TrackedDeviceUpdated => {
                    if let Some(display) = self.get_display(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Change(display.borrow().data()))
                    }
                },
                _ => {}
            };
        }
        
        result
    }
}

impl Drop for OpenVRService {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                self.gamepads.clear();
                self.displays.clear();
                println!("OpenVR Shutdown");
                (*self.lib.as_ref().unwrap().shutdown_internal)();
            }
        }
    }
}

impl OpenVRService {
    pub fn new() -> OpenVRService {
        OpenVRService {
            initialized: false,
            lib: None,
            displays: Vec::new(),
            gamepads: Vec::new(),
            system: ptr::null_mut(),
            chaperone: ptr::null_mut()
        }
    }

    fn clone_displays(&self) -> Vec<VRDisplayPtr> {
        self.displays.iter().map(|d| d.clone() as VRDisplayPtr).collect()
    }

    fn clone_gamepads(&self) -> Vec<VRGamepadPtr> {
        self.gamepads.iter().map(|d| d.clone() as VRGamepadPtr).collect()
    }

    pub fn get_display(&self, index: openvr::TrackedDeviceIndex_t) -> Option<&OpenVRDisplayPtr> {
        self.displays.iter().find(|&d| d.borrow().index() == index)
    }
}