#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRService, VRDisplay, VRDisplayPtr, VREvent, VRGamepadPtr};
use android_injected_glue as android;
use android_injected_glue::ffi as ndk;
use ovr_mobile_sys as ovr;
use std::mem;
use std::ptr;
use super::display::{OculusVRDisplay, OculusVRDisplayPtr};
use super::gamepad::{OculusVRGamepad, OculusVRGamepadPtr};
use super::jni_utils::JNIScope;

const SERVICE_CLASS_NAME:&'static str = "com/rust/webvr/OVRService";

pub struct OculusVRService {
    initialized: bool,
    ovr_java: ovr::ovrJava,
    displays: Vec<OculusVRDisplayPtr>,
    gamepads: Vec<OculusVRGamepadPtr>,
    java_object: ndk::jobject,
    java_class: ndk::jclass,
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

        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        // Return cached displays if available
        if self.is_initialized() && self.displays.len() > 0 {
            return Ok(self.clone_displays());
        }

        // Ensure that there are not initialization errors
        try!(self.initialize());
        let display = unsafe { OculusVRDisplay::new(&self.ovr_java as *const _) };
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
        Vec::new()
    }
}

impl OculusVRService {
    pub fn new() -> OculusVRService {
        OculusVRService {
            initialized: false,
            ovr_java: unsafe { mem::zeroed() },
            displays: Vec::new(),
            gamepads: Vec::new(),
            java_object: ptr::null_mut(),
            java_class: ptr::null_mut(),
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    unsafe fn api_init(&mut self) -> Result<(), String> {
        let jni_scope = try!(JNIScope::attach());

        let jni = jni_scope.jni;
        let env = jni_scope.env;

        // Use NativeActivity's classloader to find our class
        self.java_class = try!(jni_scope.find_class(SERVICE_CLASS_NAME));
        if self.java_class.is_null() {
            return Err("Didn't find GVRService class".into());
        };
        self.java_class = (jni.NewGlobalRef)(env, self.java_class);

        // Create OVRService instance and own it as a globalRef.
        let method = jni_scope.get_method(self.java_class, "create", "(Landroid/app/Activity;J)Ljava/lang/Object;", true);
        let thiz: usize = mem::transmute(self as * mut OculusVRService);
        self.java_object = (jni.CallStaticObjectMethod)(env, self.java_class, method, jni_scope.activity, thiz as ndk::jlong);
        if self.java_object.is_null() {
            return Err("Failed to create OVRService instance".into());
        };
        self.java_object = (jni.NewGlobalRef)(env, self.java_object);

        // Initialize native VrApi
        let activity: &ndk::ANativeActivity = mem::transmute(android::get_app().activity);

        self.ovr_java.Vm = mem::transmute(&mut (*jni_scope.vm).functions);
        self.ovr_java.Env = mem::transmute(&mut (*env).functions);
        self.ovr_java.ActivityObject = (jni.NewGlobalRef)(env, jni_scope.activity) as *mut _;

        let init_params = ovr::helpers::vrapi_DefaultInitParms(&self.ovr_java);
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
            unsafe {
                ovr::vrapi_Shutdown();
            }
        }
    }
}


#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_OVRService_nativeOnPause(_: *mut ndk::JNIEnv, service: ndk::jlong) {
    unsafe {
        let service: *mut OculusVRService = mem::transmute(service as usize);
        (*service).on_pause();
    }
}

#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_OVRService_nativeOnResume(_: *mut ndk::JNIEnv, service: ndk::jlong) {
    unsafe {
        let service: *mut OculusVRService = mem::transmute(service as usize);
        (*service).on_resume();
    }
}
