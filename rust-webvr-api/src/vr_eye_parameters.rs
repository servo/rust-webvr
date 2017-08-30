use VRFieldOfView;

// The VREyeParameters interface represents all the information 
// required to correctly render a scene for a given eye.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VREyeParameters {
    // Offset from the center point between the users eyes to the center of the eye in meters.
    pub offset: [f32; 3],

    // Describes the recommended render target width of each eye viewport, in pixels.
    pub render_width: u32,

    // Describes the recommended render target height of each eye viewport, in pixels.
    pub render_height: u32,

    // Describes the current field of view for the eye
    pub field_of_view: VRFieldOfView
}

impl Default for VREyeParameters {
     fn default() -> VREyeParameters {
         VREyeParameters {
             offset: [0.0, 0.0, 0.0],
             render_width: 0,
             render_height: 0,
             field_of_view: VRFieldOfView::default()
         }
     }
}