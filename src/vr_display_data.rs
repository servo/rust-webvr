use VRDisplayCapabilities;
use VREyeParameters;
use VRStageParameters;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRDisplayData {
    pub display_id: u64,
    pub display_name: String,
    pub connected: bool,
    pub capabilities: VRDisplayCapabilities,
    pub stage_parameters: Option<VRStageParameters>,
    pub left_eye_parameters: VREyeParameters,
    pub right_eye_parameters: VREyeParameters,
}

impl Default for VRDisplayData {
     fn default() -> VRDisplayData {
         VRDisplayData {
            display_id: 0,
            display_name: String::new(),
            connected: false,
            capabilities: VRDisplayCapabilities::default(),
            stage_parameters: None,
            left_eye_parameters: VREyeParameters::default(),
            right_eye_parameters: VREyeParameters::default()
         }
     }
}