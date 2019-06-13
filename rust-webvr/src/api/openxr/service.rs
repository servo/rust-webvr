use {VRService, VRDisplayPtr, VRGamepadPtr, VREvent};
use openxr::{ApplicationInfo, Entry, ExtensionSet, Instance};
use openxr::sys::platform::{HDC, HGLRC};
use std::cell::RefCell;
use std::default::Default;
use std::sync::Arc;
use super::display::OpenXrDisplay;

// OpenXr Service implementation
pub struct OpenXrService {
    instance: Option<Instance>,
    h_dc: HDC,
    h_glrc: HGLRC,
    display: Option<VRDisplayPtr>,
    events: RefCell<Vec<VREvent>>,
}

unsafe impl Send for OpenXrService {}

impl OpenXrService {
    pub fn new(h_dc: HDC, h_glrc: HGLRC) -> OpenXrService {
        OpenXrService {
            instance: None,
            h_dc,
            h_glrc,
            display: None,
            events: Default::default(),
        }
    }
}

impl VRService for OpenXrService {
    fn initialize(&mut self) -> Result<(), String> {
        if self.instance.is_some() {
            return Ok(());
        }

        let entry = Entry::load().map_err(|e| format!("{:?}", e))?;

        let app_info = ApplicationInfo {
            application_name: "webvr",
            ..Default::default()
        };

        let exts = ExtensionSet {
            khr_opengl_enable: true,
            ..Default::default()
        };

        let instance = entry
            .create_instance(&app_info, &exts)
            .map_err(|e| format!("{:?}", e))?;


        self.instance = Some(instance);

        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>, String> {
        if self.display.is_none() {
            let display = OpenXrDisplay::new(
                self.instance.as_ref().expect("uninitialized?"),
                self.h_dc, self.h_glrc,
            )?;
            self.events.borrow_mut().push(VREvent::Display(
                VRDisplayEvent::Activate(display.get_data(), VRDisplayEventReason::Mounted)
            ));
            self.display = Some(Arc::new(RefCell::new(display)));
        }
        Ok(vec![self.display.clone().unwrap()])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String> {
        Ok(vec![])
    }

    fn is_available(&self) -> bool {
        true
    }

    fn poll_events(&self) -> Vec<VREvent> {
        mem::replace(&mut *self.events.borrow_mut(), Vec::new())
    }
}
