// Information about a FBO provided by a VRDisplay.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRFramebuffer {
    // True if the framebuffer uses multiview
    pub multiview: bool,

    // UVs defining the texture bounds to present to the eye in UV space: [x,y,w,h]
    // Defaults to [0.0, 0.0, 0.5, 1.0]
    pub viewport: VRViewport,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRViewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl VRViewport {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x: x,
            y: y,
            width: width,
            height: height,
        }
    }
}