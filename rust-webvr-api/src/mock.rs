use crate::{VREyeParameters, VRStageParameters};

#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub enum MockVRControlMsg {
    SetViewerPose([f32; 3], [f32; 4]),
    SetEyeParameters(VREyeParameters, VREyeParameters),
    SetProjectionMatrices([f32; 16], [f32; 16]),
    SetStageParameters(VRStageParameters),
    Focus,
    Blur,
}
