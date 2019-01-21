#[cfg(feature = "vrexternal")]
mod vrexternal;
#[cfg(feature = "vrexternal")]
pub use self::vrexternal::{VRExternalServiceCreator, VRExternalShmemPtr};

#[cfg(feature = "mock")]
mod mock;
#[cfg(feature = "mock")]
pub use self::mock::MockServiceCreator;

#[cfg(all(target_os="windows", feature = "openvr"))]
mod openvr;
#[cfg(all(target_os="windows", feature = "openvr"))]
pub use self::openvr::OpenVRServiceCreator;

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
