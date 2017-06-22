#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRService, VRDisplayPtr, VREvent, VRGamepadPtr};
use android_injected_glue as android;
use android_injected_glue::ffi as ndk;
use ovr_mobile_sys as ovr;
use std::mem;
use std::ptr;
use super::display::{OculusVRDisplay, OculusVRDisplayPtr};
use super::jni_utils::JNIScope;

const SERVICE_CLASS_NAME:&'static str = "com/rust/webvr/OVRService"; 

pub struct OculusVRService {
    initialized: bool,
    displays: Vec<OculusVRDisplayPtr>,
    service_java: OVRServiceJava,
    ovr_java: OVRJava,
    // SurfaceView Life cycle
    resume_received: bool,
    pause_received: bool,
    surface_create_received: bool,
    surface_destroy_received: bool,
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
        let display = OculusVRDisplay::new(self.service_java.clone(), self.ovr_java.handle());
        self.displays.push(display);

        Ok(self.clone_displays())
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        try!(self.initialize());

        let mut result = Vec::new();
        for display in &self.displays {
            display.borrow().fetch_gamepads(&mut result);
        }

        let result = result.drain(0..).map(|d| d as VRGamepadPtr).collect();
        Ok(result)
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
            displays: Vec::new(),
            service_java: OVRServiceJava::default(),
            ovr_java: OVRJava::default(),
            resume_received: true, // True because Activity is already resumed when service initialized
            pause_received: false,
            surface_create_received: false,
            surface_destroy_received: false,
        }
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    unsafe fn api_init(&mut self) -> Result<(), String> {
        try!(self.ovr_java.attach());
        
        let jni_scope = self.ovr_java.jni_scope.as_ref().unwrap();
        let jni = jni_scope.jni();
        let env = jni_scope.env;
        let activity = jni_scope.activity;

        // Use NativeActivity's classloader to find our class
        let java_class = try!(jni_scope.find_class(SERVICE_CLASS_NAME));
        if java_class.is_null() { 
            return Err("Didn't find OVRService class".into());
        };

        // Create OVRService instance and own it as a globalRef.
        let method = jni_scope.get_method(java_class,
                                          "create",
                                          "(Landroid/app/Activity;J)Ljava/lang/Object;",
                                          true);
        let thiz: usize = mem::transmute(self as *const Self);
        let java_object = (jni.CallStaticObjectMethod)(env, java_class, method, activity, thiz as ndk::jlong);
        if java_object.is_null() { 
            return Err("Failed to create OVRService instance".into());
        };

        // Cache java object instances
        self.service_java.instance = (jni.NewGlobalRef)(env, java_object);
        self.service_java.class = (jni.NewGlobalRef)(env, java_class);

        // Initialize native VrApi 
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

    // Called from Java main thread
    // Pause & resume methods are thread safe
    fn on_pause(&mut self) {
        for display in &self.displays {
            unsafe {
                (*display.as_ptr()).pause();
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
    }

    // Called from Java main thread
    unsafe fn update_surface(&mut self, env: *mut ndk::JNIEnv, surface: ndk::jobject) {
        // Main thread is already attached, so JNIScope::attach() must be avoded
        let jni = JNIScope::jni_from_env(env);

        // Release previus reference if rquired
        if self.service_java.surface.is_null() {
            ((*jni).DeleteGlobalRef)(env, self.service_java.surface as *mut _);
        }

        // Protect new ref if required
        if !surface.is_null() {
            self.service_java.surface = ((*jni).NewGlobalRef)(env, surface);
        } else {
            self.service_java.surface = ptr::null_mut();
        }

        // Notify Displays
        for display in &self.displays {
            (*display.as_ptr()).update_surface(self.service_java.surface);
        }
    }

    // An Android Activity is only in the resumed state with a valid Android Surface between
    // surfaceChanged() or onResume(), whichever comes last, and surfaceDestroyed() or onPause(),
    // whichever comes first. In other words, a VR application will typically enter VR mode
    // from surfaceChanged() or onResume(), whichever comes last, and leave VR mode from
    // surfaceDestroyed() or onPause(), whichever comes first.
    fn handle_life_cycle(&mut self) {
        if self.surface_create_received && self.resume_received {
            self.on_resume();
            self.clear_flags();
        } else if self.pause_received || self.surface_destroy_received {
            self.on_pause();
            self.clear_flags();
        }
    }

    fn handle_event(&mut self, event: android::Event) {
        match event {
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

    fn clear_flags(&mut self) {
        self.resume_received = false;
        self.pause_received = false;
        self.surface_create_received = false;
        self.surface_destroy_received = false;
    }
}

impl Drop for OculusVRService {
    fn drop(&mut self) {
        if let Some(jni_scope) = self.ovr_java.jni_scope.as_ref() {
            // Delete JNI global Refs
            let jni = jni_scope.jni();
            let env = jni_scope.env;

            let surface = self.service_java.surface;
            let instance = self.service_java.instance;
            let class = self.service_java.class;

            if !surface.is_null() {
                (jni.DeleteGlobalRef)(env, surface as *mut _);
            }
            if !instance.is_null() {
                (jni.DeleteGlobalRef)(env, instance as *mut _);
            }
            if !class.is_null() {
                (jni.DeleteGlobalRef)(env, class as *mut _);
            }
        }
        if self.is_initialized() {
            // Shutdown API
            unsafe {
                ovr::vrapi_Shutdown();
            }
        }
    }
}

// Used to handle the life cycle of the ovrJava objects.
// JNI thread must be attached while the ovrJava instance is used.
pub struct OVRJava {
    pub java: ovr::ovrJava,
    pub jni_scope: Option<JNIScope>,
}

impl Default for OVRJava {
    fn default() -> OVRJava {
        OVRJava {
            java: unsafe { mem::zeroed() },
            jni_scope: None,
        }
    }
}

impl OVRJava {
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

// Stores the cached JNI objects
#[derive(Clone)]
pub struct OVRServiceJava {
    // The instance of the helper OVRService Java class
    pub instance: ndk::jobject, 
    // The cached class of the helper OVRService Java class
    pub class: ndk::jclass, 
    // The instance of the Surface used for VR rendering
    pub surface: ndk::jobject,
}

impl Default for OVRServiceJava {
    fn default() -> OVRServiceJava {
        OVRServiceJava {
            instance: ptr::null_mut(),
            class: ptr::null_mut(),
            surface: ptr::null_mut(),
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
        (*service).handle_event(android::Event::Pause);
    }
}

#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_OVRService_nativeOnResume(_: *mut ndk::JNIEnv, service: ndk::jlong) {
    unsafe {
        let service: *mut OculusVRService = mem::transmute(service as usize);
        (*service).handle_event(android::Event::Resume);
    }
}

#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_OVRService_nativeOnSurfaceChanged(env: *mut ndk::JNIEnv,
                                                                    service: ndk::jlong,
                                                                    surface: ndk::jobject) {
    unsafe {
        let service: *mut OculusVRService = mem::transmute(service as usize);
        (*service).update_surface(env, surface);
        (*service).handle_event(android::Event::InitWindow);
    }
}

#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_OVRService_nativeOnSurfaceDestroyed(env: *mut ndk::JNIEnv, service: ndk::jlong) {
    unsafe {
        let service: *mut OculusVRService = mem::transmute(service as usize);
        (*service).update_surface(env, ptr::null_mut());
        (*service).handle_event(android::Event::TermWindow);
    }
}
