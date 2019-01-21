use std::collections::HashMap;
use VRDisplayPtr;
use VREvent;
use VRGamepadPtr;
use VRService;
use VRServiceCreator;

#[cfg(target_os = "android")]
#[cfg(feature = "googlevr")]
use api::GoogleVRServiceCreator;

#[cfg(target_os = "windows")]
#[cfg(feature = "openvr")]
use api::OpenVRServiceCreator;

#[cfg(target_os = "android")]
#[cfg(feature = "oculusvr")]
use api::OculusVRServiceCreator;

#[cfg(feature = "mock")]
use api::MockServiceCreator;

#[cfg(feature = "vrexternal")]
use api::{VRExternalServiceCreator, VRExternalShmemPtr};

// Single entry point all the VRServices and displays
pub struct VRServiceManager {
    initialized: bool,
    services: Vec<Box<VRService>>,
    displays: HashMap<u32, VRDisplayPtr>,
    gamepads: HashMap<u32, VRGamepadPtr>
}

impl Drop for VRServiceManager {
     fn drop(&mut self) {
         self.gamepads.clear();
         self.displays.clear();
         self.services.clear();
     }
}

impl VRServiceManager {
    pub fn new() -> VRServiceManager {
        VRServiceManager {
            initialized: false,
            services: Vec::new(),
            displays: HashMap::new(),
            gamepads: HashMap::new()
        }
    }

    // Register default VR services specified in crate's features
    pub fn register_defaults(&mut self) {
        let creators: Vec<Box<VRServiceCreator>> = vec!(
            #[cfg(target_os = "windows")]
            #[cfg(feature = "openvr")]
            OpenVRServiceCreator::new(),
            #[cfg(target_os = "android")]
            #[cfg(feature = "googlevr")]
            GoogleVRServiceCreator::new(),
            #[cfg(target_os = "android")]
            #[cfg(feature = "oculusvr")]
            OculusVRServiceCreator::new(),
        );
        
        for creator in &creators {
            self.register(creator.new_service());
        }
    }

    // Register VRExternal service.
    #[cfg(feature = "vrexternal")]
    pub fn register_vrexternal(&mut self, ptr: VRExternalShmemPtr) {
        let creator = VRExternalServiceCreator::new(ptr);
        self.register(creator.new_service());
    }

    // Register mock VR Service
    // Usefull for testing
    #[cfg(feature = "mock")]
    pub fn register_mock(&mut self) {
        let creator = MockServiceCreator::new();
        self.register(creator.new_service());
    }


    // Register a new VR service
    pub fn register(&mut self, service: Box<VRService>) {
        self.services.push(service);
    }
    
    // Initializes all the services
    pub fn initialize_services(&mut self) {
        if self.initialized {
            return;
        }

        for service in &mut self.services {
            if let Err(msg) = service.initialize() {
                error!("Error initializing VRService: {:?}", msg);
            }
        }
        self.initialized = true;
    }

    pub fn get_displays(&mut self) -> Vec<VRDisplayPtr> {
        self.fetch_displays();
        let mut result = Vec::new();
        for (_, display) in &self.displays {
            result.push(display.clone());
        }
        // Sort by display_id to match service initialization order
        result.sort_by(|a, b| a.borrow().id().cmp(&b.borrow().id()));
        result
    }

    pub fn get_gamepads(&mut self) -> Vec<VRGamepadPtr> {
        self.fetch_gamepads();
        let mut result = Vec::new();
        for (_, gamepad) in &self.gamepads {
            result.push(gamepad.clone());
        }
        // Sort by gamepad_id to match service initialization order
        result.sort_by(|a, b| a.borrow().id().cmp(&b.borrow().id()));
        result
    }

    pub fn get_display(&self, display_id: u32) -> Option<&VRDisplayPtr> {
        self.displays.get(&display_id)
    }

    pub fn poll_events(&mut self) -> Vec<VREvent> {
        let mut events = Vec::new();
        for service in &mut self.services {
            events.append(&mut service.poll_events());
        }
        events
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}

impl VRServiceManager {
    fn fetch_displays(&mut self) {
        self.initialize_services();

        for service in &mut self.services {
            let displays = service.fetch_displays();
            if let Ok(displays) = displays {
                for display in displays {
                    let key = display.borrow().id();
                    if !self.displays.contains_key(&key) {
                        self.displays.insert(key, display.clone());
                    }
                }
            }
        }
    }

    fn fetch_gamepads(&mut self) {
        self.initialize_services();

        for service in &mut self.services {
            let gamepads = service.fetch_gamepads();
            if let Ok(gamepads) = gamepads {
                for gamepad in gamepads {
                    let key = gamepad.borrow().id();
                    if !self.gamepads.contains_key(&key) {
                        self.gamepads.insert(key, gamepad.clone());
                    }
                }
            }
        }
    }
}
