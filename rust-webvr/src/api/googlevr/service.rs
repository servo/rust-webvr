#![cfg(feature = "googlevr")]

use {VRService, VRDisplay, VRDisplayPtr, VREvent, VRGamepadPtr};
use super::display::{GoogleVRDisplay, GoogleVRDisplayPtr};
#[cfg(target_os="android")]
use rust_webvr_api::jni_utils::JNIScope;
#[cfg(target_os="android")]
use android_injected_glue::ffi as ndk;
use gvr_sys as gvr;
use std::mem;
use std::ptr;

#[cfg(target_os="android")]
const SERVICE_CLASS_NAME:&'static str = "com/rust/webvr/GVRService";

pub struct GoogleVRService {
    ctx: *mut gvr::gvr_context,
    controller_ctx: *mut gvr::gvr_controller_context,
    display: Option<GoogleVRDisplayPtr>,
    #[cfg(target_os="android")]
    pub java_object: ndk::jobject,
    #[cfg(target_os="android")]
    pub java_class: ndk::jclass,
}

unsafe impl Send for GoogleVRService {}

impl VRService for GoogleVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        if self.is_initialized() {
            return Ok(());
        }

        unsafe {
            try!(self.create_context());
            self.create_controller_context();
        }

        if self.ctx.is_null() {
            return Err("GoogleVR SDK failed to initialize".into());
        }

        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        let display = self.init_display()?;
        Ok(vec![display.clone()])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        let display = self.init_display()?;
        display.borrow_mut().fetch_gamepads()
    }

    fn is_available(&self) -> bool {
        true   
    }

    fn poll_events(&self) -> Vec<VREvent> {
        let mut events = Vec::new();
        if let Some(ref display) = self.display {
            let mut d = display.borrow_mut();
            d.poll_events(&mut events);
            if let Some(ref gp) = d.gamepad() {
                gp.borrow_mut().handle_events();
            }
        }
        events
    }
}

impl GoogleVRService {
    #[cfg(target_os="android")]
    pub fn new() -> GoogleVRService {
        GoogleVRService {
            ctx: ptr::null_mut(),
            controller_ctx: ptr::null_mut(),
            display: None,
            java_object: ptr::null_mut(),
            java_class: ptr::null_mut()
        }
    }

    #[cfg(not(target_os="android"))]
    pub fn new() -> GoogleVRService {
        GoogleVRService {
            ctx: ptr::null_mut(),
            controller_ctx: ptr::null_mut(),
            display: None,
        }
    }

    // On Android, the gvr_context must be be obtained from
    // the Java GvrLayout object via GvrLayout.getGvrApi().getNativeGvrContext()
    // Java code is implemented in GVRService. It handles the life cycle of the GvrLayout.
    // JNI code is used to comunicate with that Java code.
    #[cfg(target_os="android")]
    unsafe fn create_context(&mut self) -> Result<(), String> {
        let jni_scope = try!(JNIScope::attach());

        let jni = jni_scope.jni();
        let env = jni_scope.env;

        // Use NativeActivity's classloader to find our class
        self.java_class = try!(jni_scope.find_class(SERVICE_CLASS_NAME));
        if self.java_class.is_null() {
            return Err("Didn't find GVRService class".into());
        };
        self.java_class = (jni.NewGlobalRef)(env, self.java_class);

        // Create GVRService instance and own it as a globalRef.
        let method = jni_scope.get_method(self.java_class, "create", "(Landroid/app/Activity;J)Ljava/lang/Object;", true);
        let thiz: usize = mem::transmute(self as * mut GoogleVRService);
        self.java_object = (jni.CallStaticObjectMethod)(env, self.java_class, method, jni_scope.activity, thiz as ndk::jlong);
        if self.java_object.is_null() {
            return Err("Failed to create GVRService instance".into());
        };
        self.java_object = (jni.NewGlobalRef)(env, self.java_object);

        // Finally we have everything required to get the gvr_context pointer from java :)
        let method = jni_scope.get_method(self.java_class, "getNativeContext", "()J", false);
        let pointer = (jni.CallLongMethod)(env, self.java_object, method);
        self.ctx = pointer as *mut gvr::gvr_context;
        if self.ctx.is_null() {
            return Err("Failed to getNativeGvrContext from java GvrLayout".into());
        }

        Ok(())
    }

    #[cfg(not(target_os="android"))]
    unsafe fn create_context(&mut self) -> Result<(), String>  {
        self.ctx = gvr::gvr_create();
        Ok(())
    }

    unsafe fn create_controller_context(&mut self) {
        let options = gvr::gvr_controller_get_default_options();
        self.controller_ctx = gvr::gvr_controller_create_and_init(options, self.ctx);
        gvr::gvr_controller_resume(self.controller_ctx);
    }

    fn is_initialized(&self) -> bool {
        return !self.ctx.is_null();
    }

    fn init_display(&mut self) -> Result<&GoogleVRDisplayPtr, String> {
        self.initialize()?;

        if let Some(ref d) = self.display {
            Ok(d)
        } else {
            self.display = unsafe { Some(GoogleVRDisplay::new(self, self.ctx, self.controller_ctx)) };
            Ok(self.display.as_ref().unwrap())
        }
    }

    // Called from Java main thread
    // Pause & resume methods are thread safe
    #[cfg(target_os="android")]
    fn on_pause(&mut self) {
        if let Some(ref display) = self.display {
            unsafe {
                (*display.as_ptr()).pause();
            }
        }
    }

    // Called from Java main thread
    // Pause & resume methods are thread safe
    #[cfg(target_os="android")]
    fn on_resume(&mut self) {
        if let Some(ref display) = self.display {
            unsafe {
                (*display.as_ptr()).resume();
            }
        }
    }
}

impl Drop for GoogleVRService {
    fn drop(&mut self) {
        if !self.controller_ctx.is_null() {
            unsafe {
                gvr::gvr_controller_destroy(mem::transmute(&self.ctx));
            }
        }

        if !self.ctx.is_null() {
            unsafe {
                gvr::gvr_destroy(mem::transmute(&self.ctx));
            }
        }
    }
}


#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_GVRService_nativeOnPause(_: *mut ndk::JNIEnv, service: ndk::jlong) {
    unsafe {
        let service: *mut GoogleVRService = mem::transmute(service as usize);
        (*service).on_pause();
    }
}

#[cfg(target_os="android")]
#[no_mangle]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub extern fn Java_com_rust_webvr_GVRService_nativeOnResume(_: *mut ndk::JNIEnv, service: ndk::jlong) {
    unsafe {
        let service: *mut GoogleVRService = mem::transmute(service as usize);
        (*service).on_resume();
    }
}
