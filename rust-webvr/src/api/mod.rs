#[cfg(feature = "vrexternal")]
mod vrexternal;
#[cfg(feature = "vrexternal")]
pub use self::vrexternal::VRExternalShmemPtr;
#[cfg(all(feature = "vrexternal", target_os= "android"))]
pub use self::vrexternal::VRExternalServiceCreator;

#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use self::mock::{MockServiceCreator, MockVRControlMsg};

#[cfg(feature = "glwindow")]
mod glwindow;
#[cfg(feature = "glwindow")]
pub use self::glwindow::GlWindowVRService;

#[cfg(feature = "magicleap")]
mod magicleap;
#[cfg(feature = "magicleap")]
pub use self::magicleap::MagicLeapVRService;

#[cfg(all(target_os="windows", feature = "openvr"))]
mod openvr;
#[cfg(all(target_os="windows", feature = "openvr"))]
pub use self::openvr::OpenVRServiceCreator;

#[cfg(all(target_os="windows", feature = "openxr-api"))]
mod openxr;
#[cfg(all(target_os="windows", feature = "openxr-api"))]
pub use self::openxr::OpenXrService;

#[cfg(all(feature = "googlevr", target_os= "android"))]
mod googlevr;
#[cfg(all(feature = "googlevr", target_os= "android"))]
pub use self::googlevr::GoogleVRServiceCreator;
#[cfg(all(feature = "googlevr", target_os= "android"))]
pub use self::googlevr::jni::*;

#[cfg(all(feature = "oculusvr", target_os= "android"))]
mod oculusvr;
#[cfg(all(feature = "oculusvr", target_os= "android"))]
pub use self::oculusvr::OculusVRServiceCreator;
#[cfg(all(feature = "oculusvr", target_os= "android"))]
pub use self::oculusvr::jni::*;
