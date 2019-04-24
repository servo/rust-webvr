use egl::types::EGLContext;
use gleam::gl::Gl;
use rust_webvr_api::VRDisplayEvent;
use rust_webvr_api::VRDisplayEventReason;
use rust_webvr_api::VRDisplayPtr;
use rust_webvr_api::VREvent;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRService;
use std::cell::RefCell;
use std::mem;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use super::display::MagicLeapVRDisplay;
use super::display::MagicLeapVRDisplayPtr;
use super::heartbeat::MagicLeapVRMainThreadHeartbeat;
use super::heartbeat::MagicLeapVRMessage;
use super::magicleap_c_api::MLResult;

pub struct MagicLeapVRService {
    sender: Sender<MagicLeapVRMessage>,
    display: Option<Result<MagicLeapVRDisplayPtr, MLResult>>,
    events: RefCell<Vec<VREvent>>,
}

// This is very very unsafe, but the API requires it.
unsafe impl Send for MagicLeapVRService {}

impl VRService for MagicLeapVRService {
    fn initialize(&mut self) -> Result<(), String> {
        self.get_display()?;
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>, String> {
        Ok(vec![ self.get_display()? ])
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

impl MagicLeapVRService {
    // This function should be called from the main thread.
    pub fn new(
        display_name: String,
        egl: EGLContext,
        gl: Rc<dyn Gl>,
    ) -> Result<(MagicLeapVRService, MagicLeapVRMainThreadHeartbeat), MLResult> {
        info!("Creating VRService");
        let (sender, receiver) = channel();
        let heartbeat = MagicLeapVRMainThreadHeartbeat::new(display_name, receiver, egl, gl)?;
        let service = MagicLeapVRService {
            sender: sender,
            display: None,
            events: RefCell::new(Vec::new()),
        };
        Ok((service, heartbeat))
    }

    fn get_display(&mut self) -> Result<MagicLeapVRDisplayPtr, MLResult> {
        let sender = &self.sender;
        let events = &self.events;
        self.display.get_or_insert_with(|| {
              let (dsender, dreceiver) = channel();
            let _ = sender.send(MagicLeapVRMessage::GetDisplayData(dsender));
            let display_data = dreceiver.recv().unwrap_or(Err(MLResult::UnspecifiedFailure))?;
            let display = MagicLeapVRDisplay::new(display_data.clone(), sender.clone());
            let event = VREvent::Display(VRDisplayEvent::Activate(display_data, VRDisplayEventReason::Mounted));
            events.borrow_mut().push(event);
            Ok(Arc::new(RefCell::new(display)))
        }).clone()
    }
}
