use euclid::Angle;
use euclid::RigidTransform3D;
use euclid::Trig;
use euclid::Vector3D;
use gleam::gl;
use gleam::gl::Gl;
use rust_webvr_api::utils;
use rust_webvr_api::VRDisplay;
use rust_webvr_api::VRDisplayCapabilities;
use rust_webvr_api::VRDisplayData;
use rust_webvr_api::VREyeParameters;
use rust_webvr_api::VRFieldOfView;
use rust_webvr_api::VRFrameData;
use rust_webvr_api::VRFutureFrameData;
use rust_webvr_api::VRFramebuffer;
use rust_webvr_api::VRFramebufferAttributes;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRLayer;
use rust_webvr_api::VRViewport;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use super::heartbeat::GlWindowVRMessage;
use glutin::dpi::PhysicalSize;

// Fake a display with a distance between eyes of 5cm.
const EYE_DISTANCE: f32 = 0.05;

pub type GlWindowVRDisplayPtr = Arc<RefCell<GlWindowVRDisplay>>;

pub struct GlWindowVRDisplay {
    id: u32,
    name: String,
    size: PhysicalSize,
    sender: Sender<GlWindowVRMessage>,
    pool: ArcPool<Vec<u8>>,
}

unsafe impl Sync for GlWindowVRDisplay {}

impl Drop for GlWindowVRDisplay {
    fn drop(&mut self) {
        self.stop_present();
    }
}

impl VRDisplay for GlWindowVRDisplay {
    fn id(&self) -> u32 {
        self.id
    }

    fn data(&self) -> VRDisplayData {
        let capabilities = VRDisplayCapabilities {
            has_position: false,
            has_orientation: false,
            has_external_display: true,
            can_present: true,
            presented_by_browser: false,
            max_layers: 1,
        };

        let fov_right = GlWindowVRDisplay::fov_right(self.size).to_degrees();
        let fov_up = GlWindowVRDisplay::fov_up(self.size).to_degrees();

        let field_of_view = VRFieldOfView {
            down_degrees: fov_up,
            left_degrees: fov_right,
            right_degrees: fov_right,
            up_degrees: fov_up,
        };

        let left_eye_parameters = VREyeParameters {
            offset: [-EYE_DISTANCE / 2.0, 0.0, 0.0],
            render_width: self.size.width as u32 / 2,
            render_height: self.size.height as u32,
            field_of_view: field_of_view,
        };

        let right_eye_parameters = VREyeParameters {
            offset: [EYE_DISTANCE / 2.0, 0.0, 0.0],
            ..left_eye_parameters.clone()
        };

        VRDisplayData {
            display_id: self.id,
            display_name: self.name.clone(),
            connected: true,
            capabilities: capabilities,
            stage_parameters: None,
            left_eye_parameters: left_eye_parameters,
            right_eye_parameters: right_eye_parameters,
        }
    }

    fn immediate_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        GlWindowVRDisplay::frame_data(0.0, self.size, near, far, RigidTransform3D::identity())
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        self.immediate_frame_data(near, far)
    }

    fn reset_pose(&mut self) {}

    fn sync_poses(&mut self) {}

    fn future_frame_data(&mut self, near: f64, far: f64) -> VRFutureFrameData {
        let (resolver, result) = VRFutureFrameData::blocked();
        let _ = self.sender.send(GlWindowVRMessage::StartFrame(near, far, resolver));
        result
    }

    fn bind_framebuffer(&mut self, _eye_index: u32) {}

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        let left_viewport = VRViewport {
            x: 0,
            y: 0,
            width: (self.size.width as i32) / 2,
            height: self.size.height as i32,
        };

        let right_viewport = VRViewport {
            x: self.size.width as i32 - left_viewport.width,
            ..left_viewport
        };

        vec![
            VRFramebuffer {
                eye_index: 0,
                attributes: VRFramebufferAttributes::default(),
                viewport: left_viewport,
            },
            VRFramebuffer {
                eye_index: 1,
                attributes: VRFramebufferAttributes::default(),
                viewport: right_viewport,
            },
        ]
    }

    fn render_layer(&mut self, _layer: &VRLayer) {
        unreachable!()
    }

    fn submit_frame(&mut self) {
        unreachable!()
    }

    fn submit_layer(&mut self, gl: &Gl, layer: &VRLayer) {
        // TODO: this assumes that the current GL framebuffer contains the texture
        // TODO: what to do if the layer has no texture_size?
        if let Some((width, height)) = layer.texture_size {
            let num_bytes = (width as usize) * (height as usize) * 4;
            let mut buffer = self.pool.remove().unwrap_or_else(Vec::new);
            buffer.resize(num_bytes, 0);
            gl.read_pixels_into_buffer(
                0,
                0,
                width as gl::GLsizei,
                height as gl::GLsizei,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                &mut buffer[..],
            );
            let buffer = self.pool.add(buffer);
            let _ = self.sender.send(GlWindowVRMessage::StopFrame(width, height, buffer));
        }
    }

    fn start_present(&mut self, _attributes: Option<VRFramebufferAttributes>) {
        let _ = self.sender.send(GlWindowVRMessage::StartPresenting);
    }

    fn stop_present(&mut self) {
        let _ = self.sender.send(GlWindowVRMessage::StopPresenting);
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String> {
        Ok(vec![])
    }
}

impl GlWindowVRDisplay {
    pub(crate) fn new(
        name: String,
        size: PhysicalSize,
        sender: Sender<GlWindowVRMessage>
    ) -> GlWindowVRDisplay {
        GlWindowVRDisplay {
            id: utils::new_id(),
            name: name,
            size: size,
            sender: sender,
            pool: ArcPool::new(),
        }
    }

    fn fov_up(size: PhysicalSize) -> Angle<f64> {
        Angle::radians(f64::fast_atan2(
            2.0 * size.height as f64,
            size.width as f64,
        ))
    }

    fn fov_right(size: PhysicalSize) -> Angle<f64> {
        Angle::radians(f64::fast_atan2(
            2.0 * size.width as f64,
            size.height as f64,
        ))
    }

    fn perspective(size: PhysicalSize, near: f64, far: f64) -> [f32; 16] {
        // https://github.com/toji/gl-matrix/blob/bd3307196563fbb331b40fc6ebecbbfcc2a4722c/src/mat4.js#L1271
        let near = near as f32;
        let far = far as f32;
        let f = 1.0 / GlWindowVRDisplay::fov_up(size).radians.tan() as f32;
        let nf = 1.0 / (near - far);
        let aspect = ((size.width / 2.0) as f32) / (size.height as f32);

        // Dear rustfmt, This is a 4x4 matrix, please leave it alone. Best, ajeffrey.
        {#[rustfmt::skip] 
            return [
                f / aspect, 0.0, 0.0,                   0.0,
                0.0,        f,   0.0,                   0.0,
                0.0,        0.0, (far + near) * nf,     -1.0,
                0.0,        0.0, 2.0 * far * near * nf, 0.0,
            ];
        }
    }

    pub(crate) fn frame_data(timestamp: f64, size: PhysicalSize, near: f64, far: f64, view: RigidTransform3D<f32>) -> VRFrameData {
        let left_projection_matrix = GlWindowVRDisplay::perspective(size, near, far);
        let right_projection_matrix = left_projection_matrix.clone();

        let left_offset = RigidTransform3D::from_translation(Vector3D::new(EYE_DISTANCE / 2.0, 0.0, 0.0));
        let right_offset = RigidTransform3D::from_translation(Vector3D::new(-EYE_DISTANCE / 2.0, 0.0, 0.0));

        let left_view_matrix = view
            .post_mul(&left_offset)
            .to_transform()
            .to_row_major_array();

        let right_view_matrix = view
            .post_mul(&right_offset)
            .to_transform()
            .to_row_major_array();

        VRFrameData {
            timestamp,
            left_projection_matrix,
            right_projection_matrix,
            left_view_matrix,
            right_view_matrix,
            ..VRFrameData::default()
        }
    }
}

// A pool of Arc<T>'s.
// You can add a T into the pool, and get back an Arc<T>.
// You can request a T from the pool, if there's an Arc<T> with no other owners,
// it will be removed from the pool, unwrapped and returned.

struct ArcPool<T>(Vec<Arc<T>>);

impl<T> ArcPool<T> {
    fn new() -> ArcPool<T> {
        ArcPool(Vec::new())
    }

    fn add(&mut self, val: T) -> Arc<T> {
        let result = Arc::new(val);
        self.0.push(result.clone());
        result
    }

    fn remove(&mut self) -> Option<T> {
        let i = self.0.iter().position(|arc| Arc::strong_count(arc) == 1);
        i.and_then(|i| Arc::try_unwrap(self.0.swap_remove(i)).ok())
    }
}
