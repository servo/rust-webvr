#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
pub use self::android::*;

#[cfg(not(target_os = "android"))]
mod other;
#[cfg(not(target_os = "android"))]
pub use self::other::*;
