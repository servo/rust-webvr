use VRPose;
use std::mem;
use std::ptr;

// Represents all the information needed to render a single frame of a VR scene
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRFrameData {
    // Monotonically increasing value that allows the author 
    // to determine if position state data been updated from the hardware
    pub timestamp: f64,

    // major order column matrix describing the projection to be used for the left eye’s rendering
    pub left_projection_matrix: [f32; 16],

    // major order column matrix describing the view transform to be used for the left eye’s rendering
    pub left_view_matrix: [f32; 16],

    // major order column matrix describing the projection to be used for the right eye’s rendering
    pub right_projection_matrix: [f32; 16],

    // major order column matrix describing the view transform to be used for the right eye’s rendering
    pub right_view_matrix: [f32; 16],
 
    // VRPose containing the future predicted pose of the VRDisplay
    // when the current frame will be presented.
    pub pose: VRPose,
}

impl Default for VRFrameData {
    fn default() -> VRFrameData {
        VRFrameData {
            timestamp: 0f64,
            left_projection_matrix: identity_matrix!(),
            left_view_matrix: identity_matrix!(),
            right_projection_matrix: identity_matrix!(),
            right_view_matrix: identity_matrix!(),
            pose: VRPose::default(),
        }
    }
}

impl VRFrameData {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut vec = vec![0u8; mem::size_of::<VRFrameData>()];
        unsafe {
            ptr::copy_nonoverlapping(mem::transmute(self),
                                    vec.as_mut_ptr(),
                                    mem::size_of::<VRFrameData>());
        }
        vec
    }

    pub fn from_bytes(bytes: &[u8]) -> VRFrameData {
        unsafe {
            let mut result: VRFrameData = mem::uninitialized();
            ptr::copy_nonoverlapping(bytes.as_ptr(),
                                     mem::transmute(&mut result),
                                     mem::size_of::<VRFrameData>());

            result        
        }
    }
}