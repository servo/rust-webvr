use VRDevicePtr;
use VRServicePtr;
use VRDisplayEvent;
use api::openvr::service::OpenVRService;
use std::collections::HashMap;

// Single entry point all the VRServices and devices
pub struct VRServiceManager {
    initialized: bool,
    services: Vec<VRServicePtr>,
    devices: HashMap<u64, VRDevicePtr>,
    observer: Option<Box<Fn(VRDisplayEvent)>>
}

impl VRServiceManager {

    pub fn new() -> VRServiceManager {
        VRServiceManager {
            initialized: false,
            services: Vec::new(),
            devices: HashMap::new(),
            observer: None
        }
    }

    // Register default VR services specified in crate's features
    pub fn register_default(&mut self) {
        // TODO: add feature macro
        self.register(OpenVRService::new());
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
            match service.initialize() {
                Err(msg) => error!("Error initializing VRService: {:?}", msg),
                _ => {
                    // Set event listener for the VRService
                    let this = self as *const Self;
                    service.set_observer(Some(Box::new(move |event| {
                        unsafe { 
                            (*this).notify_event(event);
                        }
                    })));
                }
            };
        }
        self.initialized  = true;
    }

    pub fn get_devices(&mut self) -> Vec<VRDevicePtr> {
        self.fetch_devices();
        let mut result = Vec::new();
        for (_, device) in &self.devices {
            result.push(device.clone());
        }
        result
    }

    pub fn get_device(&self, device_id: u64) -> Option<&VRDevicePtr> {
        self.devices.get(&device_id)
    }

    // sets the global VRDisplay Event observer 
    pub fn set_observer(&mut self, callback: Option<Box<Fn(VRDisplayEvent)>>) {
        self.observer = callback;
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

    fn notify_event(&self, event: VRDisplayEvent) {
        if let Some(ref observer) = self.observer {
            observer(event);
        }
    }
}