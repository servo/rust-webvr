// Represents all the information needed to render a single frame of a VR scene
#[derive(Debug, Clone)]
pub struct VRFrameData {
    // Monotonically increasing value that allows the author 
    // to determine if position state data been updated from the hardware
    pub timestamp: u64,

    // major order column matrix describing the projection to be used for the left eye’s rendering
    pub left_projection_matrix: [f32; 16], 

    // major order column matrix describing the view transform to be used for the left eye’s rendering
    pub left_view_matrix: [f32; 16], 

    // major order column matrix describing the projection to be used for the right eye’s rendering
    pub right_projection_matrix: [f32; 16], 

    // major order column matrix describing the view transform to be used for the right eye’s rendering
    pub right_view_matrix: [f32; 16], 
}

impl Default for VRFrameData {
    fn default() -> VRFrameData {
        VRFrameData {
            timestamp: 0,
            left_projection_matrix: identity_matrix!(),
            left_view_matrix: identity_matrix!(),
            right_projection_matrix: identity_matrix!(),
            right_view_matrix: identity_matrix!()
        }
    }
}