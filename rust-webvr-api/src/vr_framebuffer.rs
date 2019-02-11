/// Information about a FBO provided by a VRDisplay.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRFramebuffer {
    /// Eye index of the framebuffer
    pub eye_index: u32,

    /// The attributes set up for this framebuffer
    pub attributes: VRFramebufferAttributes,

    /// The 2D rectangle that should be used to project the 3D scene
    /// to the position of the eye camera. Measured in device pixels.
    pub viewport: VRViewport,
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRFramebufferAttributes {
    pub multiview: bool,
    pub depth: bool,
    pub multisampling: bool,
}

impl Default for VRFramebufferAttributes {
     fn default() -> VRFramebufferAttributes {
         Self {
            multiview: false,
            depth: false,
            multisampling: false,
         }
     }
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
