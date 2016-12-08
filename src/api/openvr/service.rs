use super::binding as openvr;
use super::binding::EVRInitError::*;
use super::binding::EVRApplicationType::*;
use super::binding::ETrackedDeviceClass::*;
use super::binding::EVREventType::*;
use super::constants;
use super::device::{OpenVRDevice, OpenVRDevicePtr};
use super::gamepad::{OpenVRGamepad, OpenVRGamepadPtr};
use super::library::OpenVRLibrary;
use std::ffi::CString;
use std::ptr;
use std::mem;
use {VRService, VRDevice, VRDevicePtr, VRDisplayEvent, VRDisplayEventReason, VRGamepadPtr};

// OpenVR Service implementation
pub struct OpenVRService {
    initialized: bool,
    lib: Option<OpenVRLibrary>,
    devices: Vec<OpenVRDevicePtr>,
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

    fn fetch_devices(&mut self) -> Result<Vec<VRDevicePtr>,String> {
        // Return cached devices if available
        if self.initialized && self.devices.len() > 0 {
            return Ok(self.clone_devices());
        }
        // Ensure that there are not initialization errors
        try!(self.initialize());

        let max_device_count: u32 = openvr::k_unMaxTrackedDeviceCount;
        self.devices.clear();

        for i in 0..max_device_count {
            let device_class: openvr::ETrackedDeviceClass = unsafe {
                (*self.system).GetTrackedDeviceClass.unwrap()(i as openvr::TrackedDeviceIndex_t)
            };
            
            match device_class {
                ETrackedDeviceClass_TrackedDeviceClass_HMD => {
                    self.devices.push(OpenVRDevice::new(self.lib.as_ref().unwrap(), i, self.system, self.chaperone));
                },
                _ => {}
            }
        }


        Ok(self.clone_devices())
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
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
    
                        result.push(VRDisplayEvent::Activate(device.borrow().display_data(), 
                                                                   VRDisplayEventReason::Mounted));
                    }
                },
                EVREventType_VREvent_TrackedDeviceUserInteractionEnded => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
    
                        result.push(VRDisplayEvent::Deactivate(device.borrow().display_data(), 
                                                                     VRDisplayEventReason::Unmounted));
                    }
                },
                EVREventType_VREvent_TrackedDeviceActivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Connect(device.borrow().display_data()))
                    }
                },
                EVREventType_VREvent_TrackedDeviceDeactivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Disconnect(device.borrow().device_id()))
                    }
                },
                EVREventType_VREvent_DashboardActivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Blur(device.borrow().display_data()))
                    }
                },
                EVREventType_VREvent_DashboardDeactivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Focus(device.borrow().display_data()))
                    }
                },
                EVREventType_VREvent_ChaperoneDataHasChanged |
                EVREventType_VREvent_IpdChanged |
                EVREventType_VREvent_TrackedDeviceUpdated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        result.push(VRDisplayEvent::Change(device.borrow().display_data()))
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
                self.devices.clear();
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
            devices: Vec::new(),
            gamepads: Vec::new(),
            system: ptr::null_mut(),
            chaperone: ptr::null_mut()
        }
    }

    fn clone_devices(&self) -> Vec<VRDevicePtr> {
        self.devices.iter().map(|d| d.clone() as VRDevicePtr).collect()
    }

    fn clone_gamepads(&self) -> Vec<VRGamepadPtr> {
        self.gamepads.iter().map(|d| d.clone() as VRGamepadPtr).collect()
    }

    pub fn get_device(&self, index: openvr::TrackedDeviceIndex_t) -> Option<&OpenVRDevicePtr> {
        self.devices.iter().find(|&d| d.borrow().index() == index)
    }
}