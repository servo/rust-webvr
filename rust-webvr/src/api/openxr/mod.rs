//use {VRServiceCreator, VRService};

mod display;
mod service;

pub use self::service::OpenXrService;

/*pub struct OpenXrServiceCreator;

impl OpenXrServiceCreator {
    pub fn new() -> Box<VRServiceCreator> {
        Box::new(OpenXrServiceCreator)
    }
}

impl VRServiceCreator for OpenXrServiceCreator {
     fn new_service(&self) -> Box<VRService> {
         Box::new(service::OpenXrService::new())
     }
}*/
