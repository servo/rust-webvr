#![feature(custom_attribute)]
#![feature(custom_derive)]
#![cfg_attr(feature = "serde-serialization", feature(proc_macro))]
#![cfg_attr(feature = "openvr", feature(untagged_unions))]

#[macro_use]
macro_rules! identity_matrix {
    () => ([1.0, 0.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0]);
}

#[cfg(feature = "openvr")]
extern crate libloading;
#[macro_use]
extern crate log;
#[cfg(feature = "serde-serialization")]
extern crate serde;
#[cfg(feature = "serde-serialization")]
#[macro_use]
extern crate serde_derive;
extern crate time;

pub mod vr_device;
pub mod vr_service;
pub mod vr_manager;
pub mod vr_display_data;
pub mod vr_display_capabilities;
pub mod vr_eye;
pub mod vr_eye_parameters;
pub mod vr_frame_data;
pub mod vr_layer;
pub mod vr_pose;
pub mod vr_stage_parameters;
pub mod vr_event;
pub mod vr_field_view;

pub use vr_device::{VRDevice,VRDevicePtr};
pub use vr_service::{VRService,VRServiceCreator};
pub use vr_manager::VRServiceManager;
pub use vr_display_data::VRDisplayData;
pub use vr_display_capabilities::VRDisplayCapabilities;
pub use vr_eye::VREye;
pub use vr_eye_parameters::VREyeParameters;
pub use vr_frame_data::VRFrameData;
pub use vr_layer::VRLayer;
pub use vr_pose::VRPose;
pub use vr_stage_parameters::VRStageParameters;
pub use vr_event::{VRDisplayEvent, VRDisplayEventReason};
pub use vr_field_view::VRFieldOfView;

pub mod api;