// The VRStageParameters interface represents the values describing the
// stage/play area for displays that support room-scale experiences.
#[allow(unused_attributes)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VRStageParameters {
    // matrix that transforms the sitting-space view matrices of VRFrameData to standing-space.
    pub sitting_to_standing_transform: [f32; 16],
    // Width of the play-area bounds in meters.
    pub size_x: f32,
    // Depth of the play-area bounds in meters
    pub size_y: f32
}