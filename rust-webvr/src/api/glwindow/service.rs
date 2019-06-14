use gleam::gl::Gl;
use glutin::{WindowedContext, NotCurrent};
use glutin::EventsLoop;
use glutin::EventsLoopClosed;
use glutin::dpi::PhysicalSize;
use rust_webvr_api::VRDisplayPtr;
use rust_webvr_api::VREvent;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRService;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use super::display::GlWindowVRDisplay;
use super::display::GlWindowVRDisplayPtr;
use super::heartbeat::GlWindowVRMainThreadHeartbeat;
use super::heartbeat::GlWindowVRMessage;

pub struct GlWindowVRService {
    name: String,
    size: PhysicalSize,
    sender: Sender<GlWindowVRMessage>,
    display: Option<GlWindowVRDisplayPtr>,
}

// This is very very unsafe, but the API requires it.
unsafe impl Send for GlWindowVRService {}

impl VRService for GlWindowVRService {
    fn initialize(&mut self) -> Result<(), String> {
        self.get_display();
        Ok(())
    }

    fn fetch_displays(&mut self) -> Result<Vec<VRDisplayPtr>, String> {
        Ok(vec![ self.get_display().clone() ])
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String> {
        Ok(vec![])
    }

    fn is_available(&self) -> bool {
        true
    }

    fn poll_events(&self) -> Vec<VREvent> {
        vec![]
    }
}

impl GlWindowVRService {
    // This function should be called from the main thread.
    pub fn new(
        name: String,
        gl_context: WindowedContext<NotCurrent>,
	events_loop_factory: EventsLoopFactory,
        gl: Rc<dyn Gl>,
    ) -> (GlWindowVRService, GlWindowVRMainThreadHeartbeat) {
        let (sender, receiver) = channel();
        let size = gl_context.window().get_inner_size().expect("No window size");
        let hidpi = gl_context.window().get_hidpi_factor();
        let heartbeat = GlWindowVRMainThreadHeartbeat::new(receiver, gl_context, events_loop_factory, gl);
        let service = GlWindowVRService {
            name: name,
            size: size.to_physical(hidpi),
            sender: sender,
            display: None,
        };
        (service, heartbeat)
    }

    fn get_display(&mut self) -> &mut GlWindowVRDisplayPtr {
        let name = &self.name;
        let sender = &self.sender;
        let size = self.size;
        self.display.get_or_insert_with(|| {
            let display = GlWindowVRDisplay::new(name.clone(), size, sender.clone());
            Arc::new(RefCell::new(display))
        })
    }
}

pub type EventsLoopFactory = Box<Fn() -> Result<EventsLoop, EventsLoopClosed>>;
