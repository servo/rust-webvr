/// Each VR service may have some code which needs to run on the main thread,
/// for example window creation on MacOS is only supported on the main thread.
/// Implementations of this trait will usually be neither `Sync` nor `Send`.
pub trait VRMainThreadHeartbeat {
    /// Run the heartbeat on the main thread.
    fn heartbeat(&mut self);

    /// Is the heartbeat expecting to be called every frame?
    fn heart_racing(&self) -> bool;
}
