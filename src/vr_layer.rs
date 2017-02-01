// Data provided to a VRDisplay and presented in the HMD.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRLayer {
    // Source texture whose contents will be presented by the 
    // VRDisplay when VRDisplay.submitFrame() is called.
    pub texture_id: u32,

    // UVs defining the texture bounds to present to the eye in UV space: [x,y,w,h]
    // Defaults to [0.0, 0.0, 0.5, 1.0]
    pub left_bounds: [f32; 4],

    // UVs defining the texture bounds to present to the eye in UV space: [x,y,w,h]
    // Defaults to [0.5, 0.0, 0.5, 1.0]
    pub right_bounds: [f32; 4],
}

impl Default for VRLayer {
    fn default() -> VRLayer {
        VRLayer {
            texture_id: 0,
            left_bounds: [0.0, 0.0, 0.5, 1.0],
            right_bounds: [0.5, 0.0, 0.5, 1.0]
        }
    }
}