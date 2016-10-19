use {VRService, VRServicePtr, VRDevice, VRDevicePtr, VRDisplayEvent, VRDisplayEventReason};
use super::constants;
use super::device::{OpenVRDevice, OpenVRDevicePtr};
use super::openvr_sys as openvr;
use super::openvr_sys::EVRInitError::*;
use super::openvr_sys::EVRApplicationType::*;
use super::openvr_sys::ETrackedDeviceClass::*;
use super::openvr_sys::EVREventType::*;
use std::ffi::CString;
use std::sync::Arc;
use std::ptr;
use std::cell::RefCell;
use std::mem;

pub struct OpenVRService {
    initialized: bool,
    devices: Vec<OpenVRDevicePtr>,
    system: *mut openvr::VR_IVRSystem_FnTable,
    observer: Option<Box<Fn(VRDisplayEvent)>>
}

unsafe impl Send for OpenVRService {}

impl VRService for OpenVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        if self.initialized {
            return Ok(());
        }
        if !self.is_available() {
            return Err("Not available".into());
        }

        let mut error = EVRInitError_VRInitError_None;
        unsafe {
             openvr::VR_InitInternal(&mut error, EVRApplicationType_VRApplication_Scene);
        }

        if error as u32 != EVRInitError_VRInitError_None as u32 {
            return Err(format!("OpenVR Internal failed with error {}", error as u32));
        }

        error = EVRInitError_VRInitError_None;
        unsafe {
            let name = CString::new(format!("FnTable:{}",constants::IVRSYSTEM_VERSION)).unwrap();
            self.system = openvr::VR_GetGenericInterface(name.as_ptr(), &mut error)
                          as *mut openvr::VR_IVRSystem_FnTable;
            (*self.system).AcknowledgeQuit_UserPrompt.unwrap();
        }

        if error as u32 != EVRInitError_VRInitError_None as u32 {
            return Err(format!("OpenVR GetGenericInterface failed with error {}", error as u32));
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

        let max_device_count: u32 = constants::K_UNMAXTRACKEDDEVICECOUNT;

        self.devices.clear();

        for i in 0..max_device_count {
            let device_class: openvr::ETrackedDeviceClass = unsafe {
                (*self.system).GetTrackedDeviceClass.unwrap()(i as openvr::TrackedDeviceIndex_t)
            };

            match device_class {
                ETrackedDeviceClass_TrackedDeviceClass_HMD => {
                    self.devices.push(OpenVRDevice::new(self.system, i));
                },
                _ => {}
            }
            
        }

        Ok(self.clone_devices())
    }

    fn is_available(&self) -> bool {
        unsafe { 
            return openvr::VR_IsHmdPresent() > 0; 
        }
    }

    fn poll_events(&self) {
        let mut event: openvr::VREvent_t = unsafe { mem::uninitialized() };
        let size = mem::size_of::<openvr::VREvent_t>() as u32;
        while unsafe { (*self.system).PollNextEvent.unwrap()(&mut event, size) } != 0 {

            let event_type: openvr::EVREventType = unsafe { mem::transmute(event.eventType) };

            match event_type {
                EVREventType_VREvent_TrackedDeviceUserInteractionStarted => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
    
                        self.notify_event(VRDisplayEvent::Activate(device.borrow().get_display_data(), 
                                                                   VRDisplayEventReason::Mounted));
                    }
                },
                EVREventType_VREvent_TrackedDeviceUserInteractionEnded => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
    
                        self.notify_event(VRDisplayEvent::Deactivate(device.borrow().get_display_data(), 
                                                                     VRDisplayEventReason::Unmounted));
                    }
                },
                EVREventType_VREvent_TrackedDeviceActivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        self.notify_event(VRDisplayEvent::Connect(device.borrow().get_display_data()))
                    }
                },
                EVREventType_VREvent_TrackedDeviceDeactivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        self.notify_event(VRDisplayEvent::Disconnect(device.borrow().device_id()))
                    }
                },
                EVREventType_VREvent_DashboardActivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        self.notify_event(VRDisplayEvent::Blur(device.borrow().get_display_data()))
                    }
                },
                EVREventType_VREvent_DashboardDeactivated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        self.notify_event(VRDisplayEvent::Focus(device.borrow().get_display_data()))
                    }
                },
                EVREventType_VREvent_ChaperoneDataHasChanged |
                EVREventType_VREvent_IpdChanged |
                EVREventType_VREvent_TrackedDeviceUpdated => {
                    if let Some(device) = self.get_device(event.trackedDeviceIndex) {
                        self.notify_event(VRDisplayEvent::Change(device.borrow().get_display_data()))
                    }
                },
                _ => {}
            };
        } 
    }

    fn set_observer(&mut self, callback: Option<Box<Fn(VRDisplayEvent)>>) {
        self.observer = callback;
    }
}

impl Drop for OpenVRService {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                debug!("OpenVR Shutdown");
                openvr::VR_ShutdownInternal();
            }
        }
    }
}

impl OpenVRService {
    pub fn new() -> VRServicePtr {
        Arc::new(RefCell::new(OpenVRService {
            initialized: false,
            devices: Vec::new(),
            system: ptr::null_mut(),
            observer: None
        }))
    }
    fn clone_devices(&self) -> Vec<VRDevicePtr> {
        self.devices.iter().map(|d| d.clone() as VRDevicePtr).collect()
    }

    pub fn get_device(&self, index: openvr::TrackedDeviceIndex_t) -> Option<&OpenVRDevicePtr> {
        self.devices.iter().find(|&d| d.borrow().index() == index)
    }

    fn notify_event(&self, event: VRDisplayEvent) {
        if let Some(ref observer) = self.observer {
            observer(event);
        }
    }
}