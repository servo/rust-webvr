// The VRPose struct represents a sensor’s state at a given timestamp.
#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde-serialization", derive(Deserialize, Serialize))]
pub struct VRPose {
    // Position of the VRDisplay as a 3D vector.
    // May be None if the sensor is incapable of providing positional data.
    pub position: Option<[f32; 3]>,

    // Linear velocity of the sensor given in meters per second.
    // May be None if the sensor is incapable of providing linear velocity data.
    pub linear_velocity: Option<[f32; 3]>,

    // Linear acceleration of the sensor given in meters per second squared.
    // May be None if the sensor is incapable of providing linear acceleration data.
    pub linear_acceleration: Option<[f32; 3]>,

    // Orientation of the sensor as a quaternion.
    // May be None if the sensor is incapable of providing orientation.
    pub orientation: Option<[f32; 4]>,

    // Angular velocity of the sensor given in radians per second.
    // May be None if the sensor is incapable of providing angular velocity data.
    pub angular_velocity: Option<[f32; 3]>,

    // Linear acceleration of the sensor given in radians per second squared.
    // May be None if the sensor is incapable of providing angular acceleration data.
    pub angular_acceleration: Option<[f32; 3]>,
}