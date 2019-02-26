/// describes the capabilities of a VRDisplay. These are expected to be static per-device/per-user.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRDisplayCapabilities {
    /// true if the VRDisplay is capable of tracking its position.
    pub has_position: bool,

    /// true if the VRDisplay is capable of tracking its orientation.
    pub has_orientation: bool,

    /// true if the VRDisplay is separate from the device’s primary display
    pub has_external_display: bool,

    /// true if the VRDisplay is capable of presenting content to an HMD or similar device.
    pub can_present: bool,

    /// true if the VR display expects the browser to present the content.
    /// This is now deprecated, a better solution is to implement `future_frame_data`
    /// and have the future resolve when the next animation frame is ready.
    #[deprecated(since="0.10.3", note="please use `future_frame_data` instead")]
    pub presented_by_browser: bool,

    /// Indicates the maximum length of the array that requestPresent() will accept,
    /// Must be 1 if canPresent is true, 0 otherwise.
    pub max_layers: u64
}

impl Default for VRDisplayCapabilities {
    fn default() -> VRDisplayCapabilities {
	#[allow(deprecated)]
        VRDisplayCapabilities {
            has_position: false,
            has_orientation: false,
            has_external_display: false,
            can_present: false,
            presented_by_browser: false,
            max_layers: 0
        }
    }
}

