#[macro_use]
macro_rules! identity_matrix {
    () => ([1.0, 0.0, 0.0, 0.0,  0.0, 1.0, 0.0, 0.0,  0.0, 0.0, 1.0, 0.0,  0.0, 0.0, 0.0, 1.0]);
}

#[cfg(all(feature = "jni_utils", target_os = "android"))]
pub extern crate android_injected_glue;

#[cfg(feature = "utils")]
extern crate time;

#[cfg(all(feature = "jni_utils", target_os = "android"))]
pub mod jni_utils;
#[cfg(feature = "utils")]
pub mod utils;

#[cfg(feature = "serde-serialization")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "ipc")]
extern crate ipc_channel;

pub mod vr_display;
pub mod vr_service;
pub mod vr_display_data;
pub mod vr_display_capabilities;
pub mod vr_eye;
pub mod vr_eye_parameters;
pub mod vr_framebuffer;
pub mod vr_frame_data;
pub mod vr_future_frame_data;
pub mod vr_layer;
pub mod vr_pose;
pub mod vr_stage_parameters;
pub mod vr_event;
pub mod vr_field_view;
pub mod vr_gamepad;

pub use vr_display::{VRDisplay,VRDisplayPtr};
pub use vr_service::{VRService,VRServiceCreator};
pub use vr_display_data::VRDisplayData;
pub use vr_display_capabilities::VRDisplayCapabilities;
pub use vr_eye::VREye;
pub use vr_eye_parameters::VREyeParameters;
pub use vr_framebuffer::{VRFramebuffer, VRFramebufferAttributes, VRViewport};
pub use vr_frame_data::VRFrameData;
pub use vr_future_frame_data::VRFutureFrameData;
pub use vr_future_frame_data::VRResolveFrameData;
pub use vr_layer::VRLayer;
pub use vr_pose::VRPose;
pub use vr_stage_parameters::VRStageParameters;
pub use vr_event::{VREvent, VRDisplayEvent, VRDisplayEventReason, VRGamepadEvent};
pub use vr_field_view::VRFieldOfView;
pub use vr_gamepad::{VRGamepad, VRGamepadPtr, VRGamepadHand,
                     VRGamepadData, VRGamepadState, VRGamepadButton};
