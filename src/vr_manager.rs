use std::collections::HashMap;
use VRDevicePtr;
use VRDisplayEvent;
use VRGamepadPtr;
use VRService;
use VRServiceCreator;

#[cfg(feature = "openvr")]
use api::OpenVRServiceCreator;

#[cfg(feature = "mock")]
use api::MockServiceCreator;

// Single entry point all the VRServices and devices
pub struct VRServiceManager {
    initialized: bool,
    services: Vec<Box<VRService>>,
    devices: HashMap<u64, VRDevicePtr>,
    gamepads: HashMap<u64, VRGamepadPtr>
}

impl Drop for VRServiceManager {
     fn drop(&mut self) {
         self.gamepads.clear();
         self.devices.clear();
         self.services.clear();
     }
}

impl VRServiceManager {
    pub fn new() -> VRServiceManager {
        VRServiceManager {
            initialized: false,
            services: Vec::new(),
            devices: HashMap::new(),
            gamepads: HashMap::new()
        }
    }

    // Register default VR services specified in crate's features
    pub fn register_defaults(&mut self) {
        let creators: Vec<Box<VRServiceCreator>> = vec!(
            #[cfg(feature = "openvr")] OpenVRServiceCreator::new()
        );
        
        for creator in &creators {
            self.register(creator.new_service());
        }
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

    pub fn get_devices(&mut self) -> Vec<VRDevicePtr> {
        self.fetch_devices();
        let mut result = Vec::new();
        for (_, device) in &self.devices {
            result.push(device.clone());
        }
        // Sort by device_id to match service initialization order
        result.sort_by(|a, b| a.borrow().device_id().cmp(&b.borrow().device_id()));
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

    pub fn get_device(&self, device_id: u64) -> Option<&VRDevicePtr> {
        self.devices.get(&device_id)
    }

    pub fn poll_events(&mut self) -> Vec<VRDisplayEvent> {
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
    fn fetch_devices(&mut self) {
        self.initialize_services();

        for service in &mut self.services {
            let devices = service.fetch_devices();
            if let Ok(devices) = devices {
                for device in devices {
                    let key = device.borrow().device_id();
                    if !self.devices.contains_key(&key) {
                        self.devices.insert(key, device.clone());
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