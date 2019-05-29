use crate::{VREyeParameters, VRStageParameters};

pub enum MockVRControlMsg {
    SetViewerPose([f32; 3], [f32; 4]),
    SetEyeParameters(VREyeParameters, VREyeParameters),
    SetProjectionMatrices([f32; 16], [f32; 16]),
    SetStageParameters(VRStageParameters),
    Focus,
    Blur,
}
