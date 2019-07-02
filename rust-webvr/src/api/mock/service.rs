use {VRService, VRDisplayPtr, VREvent, VRGamepadPtr};
use super::display::{MockVRDisplay, MockVRDisplayPtr};
use super::{MockVRControlMsg, MockVRInit};
use std::thread;
use std::sync::mpsc::Receiver;

pub struct MockVRService {
    display: MockVRDisplayPtr,
}

unsafe impl Send for MockVRService {}

impl VRService for MockVRService {
    fn initialize(&mut self) -> Result<(), String> { 
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>,String> {
        Ok(vec![self.display.clone()])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        Ok(Vec::new())
    }

    fn is_available(&self) -> bool {
        true   
    }

    fn poll_events(&self) -> Vec<VREvent> {
        self.display.borrow().poll_events()
    }
}

impl MockVRService {
    pub fn new(init: MockVRInit) -> MockVRService {
        MockVRService {
            display: MockVRDisplay::new(init),
        }
    }

    pub fn new_with_receiver(rcv: Receiver<MockVRControlMsg>, init: MockVRInit) -> MockVRService {
        let display = MockVRDisplay::new(init);
        let state = display.borrow().state_handle();
        thread::spawn(move || {
            while let Ok(msg) = rcv.recv() {
                // The only reason we need this is that the overall display API
                // is somewhat unsound:
                // https://github.com/servo/rust-webvr/issues/18 .
                // Once that is fixed we should just have a handle to the display here
                state.lock().unwrap().handle_msg(msg);
            }
        });
        MockVRService {
            display,
        }
    }
}

