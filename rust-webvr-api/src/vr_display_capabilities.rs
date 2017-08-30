// describes the capabilities of a VRDisplay. These are expected to be static per-device/per-user.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRDisplayCapabilities {
    // true if the VRDisplay is capable of tracking its position.
    pub has_position: bool,

    // true if the VRDisplay is capable of tracking its orientation.
    pub has_orientation: bool,

    // true if the VRDisplay is separate from the device’s primary display
    pub has_external_display: bool,

    // true if the VRDisplay is capable of presenting content to an HMD or similar device.
    pub can_present: bool,

    // Indicates the maximum length of the array that requestPresent() will accept,
    // Must be 1 if canPresent is true, 0 otherwise.
    pub max_layers: u64
}

impl Default for VRDisplayCapabilities {
    fn default() -> VRDisplayCapabilities {
        VRDisplayCapabilities {
            has_position: false,
            has_orientation: false,
            has_external_display: false,
            can_present: false,
            max_layers: 0
        }
    }
}

