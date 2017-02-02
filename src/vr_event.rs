use VRDisplayData;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub enum VRDisplayEventReason {
    Navigation,
    // The VRDisplay has detected that the user has put it on.
    Mounted,

    // The VRDisplay has detected that the user has taken it off.
    Unmounted
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub enum VRDisplayEvent {
    // Indicates that a VRDisplay has been connected.
    Connect(VRDisplayData),

    // Indicates that a VRDisplay has been disconnected.
    // param: display_id
    Disconnect(u64),

    // Indicates that something has occured which suggests the VRDisplay should be presented to
    Activate(VRDisplayData, VRDisplayEventReason),

    // Indicates that something has occured which suggests the VRDisplay should exit presentation
    Deactivate(VRDisplayData, VRDisplayEventReason),

    // Indicates that some of the VRDisplay's data has changed (eye parameters, tracking data, chaperone, ipd, etc.)
    Change(VRDisplayData),

    // Indicates that presentation to the display by the page is paused by the user agent, OS, or VR hardware
    Blur(VRDisplayData),

    // Indicates that presentation to the display by the page has resumed after being blurred.
    Focus(VRDisplayData),

    // Indicates that a VRDisplay has begun or ended VR presentation
    PresentChange(VRDisplayData, bool)
}