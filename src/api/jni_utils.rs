#![cfg(any(feature = "googlevr", feature= "oculusvr"))]
#![cfg(target_os = "android")]

use android_injected_glue as android;
use android_injected_glue::ffi as ndk;
use std::ffi::CString;
use std::ptr;
use std::mem;

pub struct JNIScope<'a> {
    pub vm: &'a mut ndk::_JavaVM,
    pub env: *mut ndk::JNIEnv,
    pub jni: &'a ndk::JNINativeInterface,
    pub activity: ndk::jobject
}

impl<'a> JNIScope<'a> {
    pub unsafe fn attach() -> Result<JNIScope<'a>, String> {
        let mut env: *mut ndk::JNIEnv = mem::uninitialized();
        let activity: &ndk::ANativeActivity = mem::transmute(android::get_app().activity);
        let vm: &mut ndk::_JavaVM = mem::transmute(activity.vm);
        let vmf: &ndk::JNIInvokeInterface = mem::transmute(vm.functions);

        // Attach is required because native_glue is running in a separate thread
        if (vmf.AttachCurrentThread)(vm as *mut _, &mut env as *mut _, ptr::null_mut()) != 0 {
            return Err("JNI AttachCurrentThread failed".into());
        }

        let jni: &ndk::JNINativeInterface = mem::transmute((*env).functions);

        Ok(JNIScope {
            vm: vm,
            env: env,
            jni: jni,
            activity: activity.clazz
        })
    }

    pub unsafe fn find_class(&self, class_name: &str) -> Result<ndk::jclass, String> {
        // jni.FindClass doesn't find our classes because the attached thread has not our classloader.
        // NativeActivity's classloader is used to fix this issue.
        let env = self.env;
        let jni = self.jni;

        let activity_class = (jni.GetObjectClass)(env, self.activity);
        if activity_class.is_null() {
            return Err("Didn't find NativeActivity class".into());
        }
        let method = self.get_method(activity_class, "getClassLoader", "()Ljava/lang/ClassLoader;", false);
        let classloader = (jni.CallObjectMethod)(env, self.activity, method);
        if classloader.is_null() {
            return Err("Didn't find NativeActivity's classloader".into());
        }
        let classloader_class = (jni.GetObjectClass)(env, classloader);
        let load_method = self.get_method(classloader_class, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", false);

        // Load our class using the classloader.
        let class_name = CString::new(class_name).unwrap();
        let class_name = (jni.NewStringUTF)(env, class_name.as_ptr());
        let java_class = (jni.CallObjectMethod)(env, classloader, load_method, class_name) as ndk::jclass;
        (jni.DeleteLocalRef)(env, class_name);

        Ok(java_class)
    }

    pub unsafe fn get_method(&self,
                             class: ndk::jclass,
                             method: &str,
                             signature: &str,
                             is_static: bool) -> ndk::jmethodID {
        let method = CString::new(method).unwrap();
        let signature = CString::new(signature).unwrap();

        if is_static {
            (self.jni.GetStaticMethodID)(self.env, class, method.as_ptr(), signature.as_ptr())
        } else {
            (self.jni.GetMethodID)(self.env, class, method.as_ptr(), signature.as_ptr())
        }
    } 
}

impl<'a> Drop for JNIScope<'a> {
    // Autodetach JNI thread
    fn drop(&mut self) {
        unsafe {
            let vmf: &ndk::JNIInvokeInterface = mem::transmute(self.vm.functions);
            (vmf.DetachCurrentThread)(self.vm);
        }
    }
}
