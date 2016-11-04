use std::collections::HashMap;
use VRDevicePtr;
use VRDisplayEvent;
use VRServicePtr;

#[cfg(feature = "openvr")]
use api::openvr::service::OpenVRService;

#[cfg(feature = "mock")]
use api::mock::service::MockVRService;

// Single entry point all the VRServices and devices
pub struct VRServiceManager {
    initialized: bool,
    services: Vec<VRServicePtr>,
    devices: HashMap<u64, VRDevicePtr>
}

impl VRServiceManager {

    pub fn new() -> VRServiceManager {
        VRServiceManager {
            initialized: false,
            services: Vec::new(),
            devices: HashMap::new()
        }
    }

    // Register default VR services specified in crate's features
    pub fn register_defaults(&mut self) {

        let services: Vec<VRServicePtr> = vec!(
            #[cfg(feature = "openvr")] OpenVRService::new()
        );
        
        for service in &services {
            self.register(service.clone());
        }
    }

    // Register mock VR Service
    // Usefull for testing
    #[cfg(feature = "mock")]
    pub fn register_mock(&mut self) {
        self.register(MockVRService::new());
    }


    // Register a new VR service
    pub fn register(&mut self, service: VRServicePtr) {
        self.services.push(service.clone());
    }
    
    // Initializes all the services
    pub fn initialize_services(&mut self) {
        if self.initialized {
            return;
        }

        for service in &self.services {
            let mut service = service.borrow_mut();
            if let Err(msg) = service.initialize() {
                error!("Error initializing VRService: {:?}", msg);
            }
        }
        self.initialized  = true;
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

    pub fn get_device(&self, device_id: u64) -> Option<&VRDevicePtr> {
        self.devices.get(&device_id)
    }

    pub fn poll_events(&self) -> Vec<VRDisplayEvent> {
        let mut events = Vec::new();
        for service in &self.services {
            events.append(&mut service.borrow().poll_events());
        }
        events
    }
}

impl VRServiceManager {
    fn fetch_devices(&mut self) {
        self.initialize_services();

        for service in &self.services {
            let devices = service.borrow_mut().fetch_devices();
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
}