#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use libc;

include!(concat!(env!("OUT_DIR"), "/moz_external_vr.rs"));

pub type PthreadResult = Result<(), i32>;

impl VRExternalShmem {
    pub fn pull_system(&mut self, exit_cond: &Fn(&VRSystemState) -> bool) -> VRSystemState {
        self.systemMutex.lock().expect("systemMutex lock error");
        loop {
            if exit_cond(&self.state) {
                break;
            }
            self.systemCond.wait(&mut self.systemMutex).expect("systemCond wait error");
        }
        let state = self.state.clone();
        self.systemMutex.unlock().expect("systemMutex unlock error");
        state
    }
    pub fn pull_browser(&mut self) -> VRBrowserState {
        self.servoMutex.lock().expect("servoMutex lock error");
        let state = self.servoState.clone();
        self.servoMutex.unlock().expect("servoMutex unlock error");
        state
    }
    pub fn push_browser(&mut self, state: VRBrowserState) {
        self.servoMutex.lock().expect("servoMutex lock error");
        self.servoState = state;
        self.servoCond.signal().expect("servoCond signal error");
        self.servoMutex.unlock().expect("servoMutex unlock error");
    }
}

impl pthread_mutex_t {
    fn as_libc(&mut self) -> *mut libc::pthread_mutex_t {
        self as *mut _ as *mut libc::pthread_mutex_t
    }
    pub fn lock(&mut self) -> PthreadResult {
        let r = unsafe { libc::pthread_mutex_lock(self.as_libc()) };
        if r == 0 { Ok(()) } else { Err(r) }
    }
    pub fn unlock(&mut self) -> PthreadResult {
        let r = unsafe { libc::pthread_mutex_unlock(self.as_libc()) };
        if r == 0 { Ok(()) } else { Err(r) }
    }
}

impl pthread_cond_t {
    fn as_libc(&mut self) -> *mut libc::pthread_cond_t {
        self as *mut _ as *mut libc::pthread_cond_t
    }
    pub fn wait(&mut self, mutex: &mut pthread_mutex_t) -> PthreadResult {
        let r = unsafe { libc::pthread_cond_wait(self.as_libc(), mutex.as_libc()) };
        if r == 0 { Ok(()) } else { Err(r) }
    }
    pub fn signal(&mut self) -> PthreadResult {
        let r = unsafe { libc::pthread_cond_signal(self.as_libc()) };
        if r == 0 { Ok(()) } else { Err(r) }
    }
}
