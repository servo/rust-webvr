
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub enum MockVRControlMsg {
    SetViewerPose([f32; 3], [f32; 4]),
    SetViews(MockVRView, MockVRView),
    SetEyeLevel(f32),
    Focus,
    Blur,
}

#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
#[derive(Debug, Default)]
pub struct MockVRInit {
    pub views: Option<(MockVRView, MockVRView)>,
    pub eye_level: Option<f32>,
    pub viewer_origin: Option<([f32; 3], [f32; 4])>,
}

#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
#[derive(Debug)]
pub struct MockVRView {
    pub projection: [f32; 16],
    pub offset: [f32; 3],
}
