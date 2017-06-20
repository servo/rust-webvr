#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRService, VRDisplay, VRDisplayPtr, VREvent, VRGamepadPtr};
use android_injected_glue as android;
use ovr_mobile_sys as ovr;
use std::mem;
use std::ptr;
use super::display::{OculusVRDisplay, OculusVRDisplayPtr};
use super::gamepad::{OculusVRGamepad, OculusVRGamepadPtr};
use super::jni_utils::JNIScope;

pub struct OculusVRService {
    initialized: bool,
    ovr_java: OVRJava,
    displays: Vec<OculusVRDisplayPtr>,
    gamepads: Vec<OculusVRGamepadPtr>,
    android_event_handler: *const AndroidEventHandler, 
}

unsafe impl Send for OculusVRService {}

impl VRService for OculusVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        if self.is_initialized() {
            return Ok(());
        }

        unsafe {
            try!(self.api_init());
            //self.create_controller_context();
        }

        // Register Android Event Handler
        let handler = Box::new(AndroidEventHandler::new(self));
        self.android_event_handler = handler.as_ref() as *const _;
        android::add_sync_event_handler(handler);

        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        // Return cached displays if available
        if self.is_initialized() && self.displays.len() > 0 {
            return Ok(self.clone_displays());
        }

        // Ensure that there are not initialization errors
        try!(self.initialize());
        let display = OculusVRDisplay::new(self.ovr_java.handle());
        self.displays.push(display);

        Ok(self.clone_displays())
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        // Return cached gamepads if available
        if self.is_initialized() && self.gamepads.len() > 0 {
            return Ok(self.clone_gamepads());
        }
        try!(self.initialize());

        let gamepad = unsafe {
            let display_id = match self.displays.first() {
                Some(display) => display.borrow().id(),
                None => 0
            };
            try!(OculusVRGamepad::new(display_id))
        };
        self.gamepads.push(gamepad);
        
        Ok(self.clone_gamepads())
    }

    fn is_available(&self) -> bool {
        true
    }

    fn poll_events(&self) -> Vec<VREvent> {
        let mut events = Vec::new();
        for display in &self.displays {
            display.borrow_mut().poll_events(&mut events);
        }
        events
    }
}

impl OculusVRService {
    pub fn new() -> OculusVRService {
        OculusVRService {
            initialized: false,
            ovr_java: OVRJava::empty(),
            displays: Vec::new(),
            gamepads: Vec::new(),
            android_event_handler: ptr::null(),
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    unsafe fn api_init(&mut self) -> Result<(), String> {
        try!(self.ovr_java.attach());
        let init_params = ovr::helpers::vrapi_DefaultInitParms(self.ovr_java.handle());
        let status = ovr::vrapi_Initialize(&init_params);

        if status == ovr::ovrInitializeStatus::VRAPI_INITIALIZE_SUCCESS {
            self.initialized = true;
            Ok(())
        } else {
            Err(format!("OVR failed to initialize: {:?}", status))
        }
    }


    fn clone_displays(&self) -> Vec<VRDisplayPtr> {
        self.displays.iter().map(|d| d.clone() as VRDisplayPtr).collect()
    }

    fn clone_gamepads(&self) -> Vec<VRGamepadPtr> {
        self.gamepads.iter().map(|d| d.clone() as VRGamepadPtr).collect()
    }

    // Called from Java main thread
    // Pause & resume methods are thread safe
    fn on_pause(&mut self) {
        for display in &self.displays {
            unsafe {
                (*display.as_ptr()).pause();
            }
        }

        for gamepad in &self.gamepads {
            unsafe {
                (*gamepad.as_ptr()).pause();
            }
        }
    }

    // Called from Java main thread
    // Pause & resume methods are thread safe
    fn on_resume(&mut self) {
        for display in &self.displays {
            unsafe {
                (*display.as_ptr()).resume();
            }
        }
        for gamepad in &self.gamepads {
            unsafe {
                (*gamepad.as_ptr()).resume();
            }
        }
    }
}

impl Drop for OculusVRService {
    fn drop(&mut self) {
        if self.is_initialized() {
            // Unregister Android Event Handler
            android::remove_sync_event_handler(self.android_event_handler);

            // Shutdown API
            unsafe {
                ovr::vrapi_Shutdown();
            }
        }
    }
}


struct AndroidEventHandler {
    service: *mut OculusVRService,
    resume_received: bool,
    pause_received: bool,
    surface_create_received: bool,
    surface_destroy_received: bool,
}

impl AndroidEventHandler {
    pub fn new(service: *mut OculusVRService) -> Self {
        Self {
            service: service,
            resume_received: false,
            pause_received: false,
            surface_create_received: false,
            surface_destroy_received: false,
        }
    }

    fn clear_flags(&mut self) {
        self.resume_received = false;
        self.pause_received = false;
        self.surface_create_received = false;
        self.surface_destroy_received = false;
    }

    // An Android Activity is only in the resumed state with a valid Android Surface between
    // surfaceChanged() or onResume(), whichever comes last, and surfaceDestroyed() or onPause(),
    // whichever comes first. In other words, a VR application will typically enter VR mode
    // from surfaceChanged() or onResume(), whichever comes last, and leave VR mode from
    // surfaceDestroyed() or onPause(), whichever comes first.
    fn handle_life_cycle(&mut self) {
        if self.surface_create_received && self.resume_received {
            unsafe {
                (*self.service).on_resume();
            }
            self.clear_flags();
        } else if self.pause_received || self.surface_destroy_received {
            unsafe {
                (*self.service).on_pause();
            }
            self.clear_flags();
        }
    }
}

impl android::SyncEventHandler for AndroidEventHandler {
    fn handle(&mut self, event: &android::Event) {
        match *event {
            android::Event::InitWindow => {
                self.surface_create_received = true;
                self.handle_life_cycle();
            },
            android::Event::TermWindow => {
                self.surface_destroy_received = true;
                self.handle_life_cycle();
            },
            android::Event::Pause => {
                self.pause_received = true;
                self.handle_life_cycle();
            },
            android::Event::Resume => {
                self.resume_received = true;
                self.handle_life_cycle();
            },
            _ => {}
        }
    }
}

pub struct OVRJava {
    java: ovr::ovrJava,
    jni_scope: Option<JNIScope>,
}

impl OVRJava {
    pub fn empty() -> OVRJava {
        OVRJava {
            java: unsafe { mem::zeroed() },
            jni_scope: None,
        }
    }

    pub fn attach(&mut self) -> Result<(), String> {
        self.detach();
        unsafe {
            let jni_scope = try!(JNIScope::attach());
            {
                let jni = jni_scope.jni();
                let env = jni_scope.env;

                // Initialize native VrApi
                self.java.Vm = mem::transmute(&mut (*jni_scope.vm).functions);
                self.java.Env = mem::transmute(&mut (*env).functions);
                self.java.ActivityObject = (jni.NewGlobalRef)(env, jni_scope.activity) as *mut _;
            }
            self.jni_scope = Some(jni_scope);

            Ok(())
        }
    }

    pub fn detach(&mut self) {
        if !self.java.ActivityObject.is_null() {
            let jni_scope = self.jni_scope.as_ref().unwrap();
            let jni = jni_scope.jni();
            let env = jni_scope.env;
            (jni.DeleteGlobalRef)(env, self.java.ActivityObject as *mut _);
            self.java.ActivityObject = ptr::null_mut();
        }

        self.jni_scope = None;
    }

    pub fn handle(&self) -> *const ovr::ovrJava {
        &self.java as *const _
    }
}

impl<'a> Drop for OVRJava {
    fn drop(&mut self) {
        self.detach();
    }
}