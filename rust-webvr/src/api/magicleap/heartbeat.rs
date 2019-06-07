use egl::types::EGLContext;
use euclid::Angle;
use euclid::Rotation3D;
use euclid::Vector3D;
use euclid::RigidTransform3D;
use gleam::gl::Gl;
use gleam::gl;
use gleam::gl::types::GLuint;
use gleam::gl::types::GLint;
use rust_webvr_api::utils;
use rust_webvr_api::VRDisplayCapabilities;
use rust_webvr_api::VRDisplayData;
use rust_webvr_api::VREyeParameters;
use rust_webvr_api::VRFieldOfView;
use rust_webvr_api::VRFrameData;
use rust_webvr_api::VRLayer;
use rust_webvr_api::VRPose;
use rust_webvr_api::VRResolveFrameData;
use rust_webvr_api::VRMainThreadHeartbeat;
use std::mem;
use std::rc::Rc;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::TryRecvError;
use std::sync::mpsc::RecvTimeoutError;
use super::display::PooledGLTextureId;
use super::magicleap_c_api::MLHandle;
use super::magicleap_c_api::MLGraphicsBeginFrame;
use super::magicleap_c_api::MLGraphicsEndFrame;
use super::magicleap_c_api::MLGraphicsOptions;
use super::magicleap_c_api::MLGraphicsCreateClientGL;
use super::magicleap_c_api::MLGraphicsDestroyClient;
use super::magicleap_c_api::MLGraphicsFlags_MLGraphicsFlags_Default;
use super::magicleap_c_api::MLGraphicsFrameParams;
use super::magicleap_c_api::MLGraphicsGetRenderTargets;
use super::magicleap_c_api::MLGraphicsInitFrameParams;
use super::magicleap_c_api::MLGraphicsRenderBufferInfo;
use super::magicleap_c_api::MLGraphicsRenderTargetsInfo;
use super::magicleap_c_api::MLGraphicsSignalSyncObjectGL;
use super::magicleap_c_api::MLGraphicsVirtualCameraInfo;
use super::magicleap_c_api::MLGraphicsVirtualCameraInfoArray;
use super::magicleap_c_api::MLHeadTrackingCreate;
use super::magicleap_c_api::MLHeadTrackingDestroy;
use super::magicleap_c_api::MLHeadTrackingGetStaticData;
use super::magicleap_c_api::MLHeadTrackingStaticData;
use super::magicleap_c_api::MLLifecycleSetReadyIndication;
use super::magicleap_c_api::MLPerceptionGetSnapshot;
use super::magicleap_c_api::MLResult;
use super::magicleap_c_api::MLSnapshotGetTransform;
use super::magicleap_c_api::MLSurfaceFormat_MLSurfaceFormat_RGBA8UNormSRGB;
use super::magicleap_c_api::MLSurfaceFormat_MLSurfaceFormat_D32Float;
use super::magicleap_c_api::MLTransform;

const TIMEOUT: Duration = Duration::from_millis(16);
const POLL_INTERVAL: Duration = Duration::from_micros(500);

const DEFAULT_NEAR: f32 = 0.35;
const DEFAULT_FAR: f32 = 10.0;

// TODO: get the eye distance from the device
const EYE_DISTANCE: f32 = 0.02;

pub struct MagicLeapVRMainThreadHeartbeat {
    display_name: String,
    receiver: Receiver<MagicLeapVRMessage>,
    gl: Rc<dyn Gl>,
    read_fbo: GLuint,
    draw_fbo: GLuint,
    presenting: bool,
    in_frame: bool,
    pose: VRPose,
    graphics_client: MLHandle,
    head_tracking: MLHandle,
    head_tracking_sdata: MLHeadTrackingStaticData,
    frame_handle: MLHandle,
    cameras: MLGraphicsVirtualCameraInfoArray,
    timestamp: f64,
}

impl VRMainThreadHeartbeat for MagicLeapVRMainThreadHeartbeat {
    fn heartbeat(&mut self) {
        debug!("VR heartbeat start");
        if let Err(err) = self.handle_msgs() {
            error!("MLResult {}", String::from(err));
        }
        debug!("VR heartbeat stop");
    }

    fn heart_racing(&self) -> bool {
        self.presenting
    }
}

impl MagicLeapVRMainThreadHeartbeat {
    pub(crate) fn new(
        display_name: String,
        receiver: Receiver<MagicLeapVRMessage>,
        egl_context: EGLContext,
        gl: Rc<Gl>,
    ) -> Result<MagicLeapVRMainThreadHeartbeat, MLResult> {
        info!("Creating VRMainThreadHeartbeat");
        let options = MLGraphicsOptions {
            color_format: MLSurfaceFormat_MLSurfaceFormat_RGBA8UNormSRGB,
            depth_format: MLSurfaceFormat_MLSurfaceFormat_D32Float,
            graphics_flags: MLGraphicsFlags_MLGraphicsFlags_Default,
        };
        let mut graphics_client = MLHandle::default();
        unsafe { MLGraphicsCreateClientGL(&options, egl_context as MLHandle, &mut graphics_client).ok()? };

        let mut head_tracking = MLHandle::default();
        unsafe { MLHeadTrackingCreate(&mut head_tracking).ok()? };

        let mut head_tracking_sdata = MLHeadTrackingStaticData::default();
        unsafe { MLHeadTrackingGetStaticData(head_tracking, &mut head_tracking_sdata).ok()? };

        let framebuffers = gl.gen_framebuffers(2);
        let draw_fbo = framebuffers[0];
        let read_fbo = framebuffers[1];

        Ok(MagicLeapVRMainThreadHeartbeat {
            display_name,
            receiver,
            gl,
            graphics_client,
            head_tracking,
            head_tracking_sdata,
            draw_fbo,
            read_fbo,
            pose: VRPose::default(),
            presenting: false,
            in_frame: false,
            frame_handle: MLHandle::default(),
            cameras: MLGraphicsVirtualCameraInfoArray::default(),
            timestamp: 0.0,
        })
    }

    fn handle_msgs(&mut self) -> Result<(), MLResult> {
        // This should use self.receiver.recv_timeout(timeout) but that hits
        // https://github.com/rust-lang/rust/issues/39364
        let expire = Instant::now() + TIMEOUT;
        while let Ok(msg) = self.receiver_recv_deadline(expire) {
            match msg {
                MagicLeapVRMessage::GetDisplayData(sender) => {
                    let _ = sender.send(self.get_display_data());
                },
                MagicLeapVRMessage::StartPresenting => {
                    info!("VR starting");
                    self.start_presenting()?;
                },
                MagicLeapVRMessage::StartFrame(near, far, mut resolver) => {
                    debug!("VR start frame");
                    let data = self.start_frame(near, far)?;
                    let _ = resolver.resolve(data);
                },
                MagicLeapVRMessage::StopFrame(layer, pooled_id) => {
                    debug!("VR stop frame ({:?}, {})", layer, pooled_id.texture_id());
                    self.stop_frame(layer, pooled_id)?;
                    break;
                },
                MagicLeapVRMessage::StopPresenting => {
                    info!("VR stopping");
                    self.presenting = false;
                    break;
                },
            }
        }
        Ok(())
    }

    fn receiver_recv_deadline(&mut self, deadline: Instant) -> Result<MagicLeapVRMessage, RecvTimeoutError> {
        // Sigh, polling
        while deadline > Instant::now() {
            match self.receiver.try_recv() {
                Ok(msg) => return Ok(msg),
                Err(TryRecvError::Empty) => thread::sleep(POLL_INTERVAL),
                Err(TryRecvError::Disconnected) => return Err(RecvTimeoutError::Disconnected),
            }
        }
        Err(RecvTimeoutError::Timeout)
    }

    fn start_presenting(&mut self) -> Result<(), MLResult> {
        self.presenting = true;
        unsafe { MLLifecycleSetReadyIndication().ok()? };
        Ok(())
    }

    fn start_frame(&mut self, near_clip: f32, far_clip: f32) -> Result<VRFrameData, MLResult> {
        if !self.in_frame {
            let mut params = MLGraphicsFrameParams::default();
            unsafe { MLGraphicsInitFrameParams(&mut params).ok()? };
            params.near_clip = near_clip;
            params.far_clip = far_clip;

            let mut result = unsafe { MLGraphicsBeginFrame(self.graphics_client, &params, &mut self.frame_handle, &mut self.cameras) };
            if result == MLResult::Timeout {
                debug!("MLGraphicsBeginFrame timeout");
                let mut sleep = Duration::from_millis(1);
                let max_sleep = Duration::from_secs(5);
                // TODO: give up after a while
                while result == MLResult::Timeout {
                    sleep = (sleep * 2).min(max_sleep);
                    debug!("MLGraphicsBeginFrame exponential backoff {}ms", sleep.as_millis());
                    thread::sleep(sleep);
                    result = unsafe { MLGraphicsBeginFrame(self.graphics_client, &params, &mut self.frame_handle, &mut self.cameras) };
                }
                debug!("MLGraphicsBeginFrame finished timeout");
            }
            result.ok()?;

            let mut snapshot = unsafe { mem::zeroed() };
            unsafe { MLPerceptionGetSnapshot(&mut snapshot).ok()? };

            let mut transform = MLTransform::default();
            unsafe { MLSnapshotGetTransform(snapshot, &self.head_tracking_sdata.coord_frame_head, &mut transform).ok()? };

            self.pose.orientation = Some(unsafe { transform.rotation.__bindgen_anon_1.values });
            self.pose.position = Some(unsafe { transform.position.__bindgen_anon_1.values });
            self.timestamp = self.timestamp + 1.0;
            self.in_frame = true;
        }

        Ok(VRFrameData {
            left_projection_matrix: self.projection_matrix(0),
            left_view_matrix: self.view_matrix(0),
            right_projection_matrix: self.projection_matrix(1),
            right_view_matrix: self.view_matrix(1),
            pose: self.pose,
            timestamp: self.timestamp,
        })
    }

    fn projection_matrix(&self, index: usize) -> [f32; 16] {
        self.cameras.virtual_cameras[index].projection.matrix_colmajor
    }

    fn view_matrix(&self, index: usize) -> [f32; 16] {
        let quat = unsafe { self.cameras.virtual_cameras[index].transform.rotation.__bindgen_anon_1.values };
        let rotation = Rotation3D::quaternion(quat[0], quat[1], quat[2], quat[3]);

        let pos = unsafe { self.cameras.virtual_cameras[index].transform.position.__bindgen_anon_1.values };
        let translation = Vector3D::new(pos[0], pos[1], pos[2]);

        let transform = RigidTransform3D::new(rotation, translation);
        transform.to_transform().to_column_major_array()
    }

    fn stop_frame(&mut self, layer: VRLayer, pooled_id: PooledGLTextureId) -> Result<(), MLResult> {
        if self.in_frame {
            let mut current_fbos = [0, 0];
            unsafe { self.gl.get_integer_v(gl::DRAW_FRAMEBUFFER_BINDING, &mut current_fbos[0..]) };
            unsafe { self.gl.get_integer_v(gl::READ_FRAMEBUFFER_BINDING, &mut current_fbos[1..]) };

            self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, self.read_fbo);
            self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, self.draw_fbo);
            self.gl.framebuffer_texture_2d(gl::READ_FRAMEBUFFER,
                                           gl::COLOR_ATTACHMENT0,
                                           gl::TEXTURE_2D,
                                           pooled_id.texture_id(), 0);

            let color_id = self.cameras.color_id as GLuint;
            let depth_id = self.cameras.depth_id as GLuint;
            let draw = self.cameras.viewport;
            let (draw_x, draw_y, draw_w, draw_h) = (draw.x as GLint, draw.y as GLint, draw.w as GLint, draw.h as GLint);

            let (texture_w, texture_h) = match layer.texture_size {
                Some((w, h)) => (w as f32, h as f32),
                None => (draw.w * 2.0, draw.h),
            };

            for i in 0..self.cameras.num_virtual_cameras {
                let bounds = if i == 0 { &layer.left_bounds } else { &layer.right_bounds };
                let read_x = (bounds[0] * texture_w) as GLint;
                let read_y = (bounds[1] * texture_h) as GLint;
                let read_w = (bounds[2] * texture_w) as GLint;
                let read_h = (bounds[3] * texture_h) as GLint;
                let camera = &self.cameras.virtual_cameras[i as usize];
                let layer_id = camera.virtual_camera_name;
                self.gl.framebuffer_texture_layer(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, color_id, 0, layer_id);
                self.gl.framebuffer_texture_layer(gl::DRAW_FRAMEBUFFER, gl::DEPTH_ATTACHMENT, depth_id, 0, layer_id);
                self.gl.viewport(draw_x, draw_y, draw_w, draw_h);
                self.gl.blit_framebuffer(read_x, read_y, read_x + read_w, read_y + read_h,
                                         draw_x, draw_y, draw_x + draw_w, draw_y + draw_h,
                                         gl::COLOR_BUFFER_BIT, gl::LINEAR);
                unsafe { MLGraphicsSignalSyncObjectGL(self.graphics_client, camera.sync_object).ok()? };
            }

            unsafe { MLGraphicsEndFrame(self.graphics_client, self.frame_handle).ok()? };

            self.gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, current_fbos[0] as GLuint);
            self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, current_fbos[1] as GLuint);

            self.in_frame = false;

            // At this point it is safe to drop the texture and put it back into the pool
            // TODO: make sure that MLGraphicsSignalSyncObjectGL and MLGraphicsEndFrame do enough sync'ing
            drop(pooled_id);
        }

        Ok(())
    }

    fn get_display_data(&mut self) -> Result<VRDisplayData, MLResult> {
        let mut targets = MLGraphicsRenderTargetsInfo::default();
        unsafe { MLGraphicsGetRenderTargets(self.graphics_client, &mut targets).ok()? };

        // Rather annoyingly, the ML1 API only allows access to the FOV during a frame
        self.start_frame(DEFAULT_NEAR, DEFAULT_FAR)?;

        let left_eye_parameters = Self::eye_parameters(&targets.buffers[0], &self.cameras.virtual_cameras[0], -EYE_DISTANCE);
        let right_eye_parameters = Self::eye_parameters(&targets.buffers[1], &self.cameras.virtual_cameras[1], EYE_DISTANCE);
        let display_name = self.display_name.clone();
        let capabilities = VRDisplayCapabilities {
            has_position: true,
            has_orientation: true,
            has_external_display: false,
            can_present: true,
            presented_by_browser: false,
            max_layers: 1,
        };
        Ok(VRDisplayData {
            display_name,
            capabilities,
            left_eye_parameters,
            right_eye_parameters,
            connected: true,
            stage_parameters: None,
            display_id: utils::new_id(),
        })
    }

    fn eye_parameters(
        buffer: &MLGraphicsRenderBufferInfo,
        camera: &MLGraphicsVirtualCameraInfo,
        distance: f32,
    ) -> VREyeParameters {
        VREyeParameters {
            field_of_view: VRFieldOfView {
                down_degrees: Angle::radians(camera.bottom_half_angle as f64).to_degrees(),
                up_degrees: Angle::radians(camera.top_half_angle as f64).to_degrees(),
                left_degrees: Angle::radians(camera.left_half_angle as f64).to_degrees(),
                right_degrees: Angle::radians(camera.right_half_angle as f64).to_degrees(),
            },
            offset: [distance, 0.0, 0.0],
            render_height: buffer.color.height,
            render_width: buffer.color.width,
        }
    }
}

impl Drop for MagicLeapVRMainThreadHeartbeat {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_framebuffers(&[self.draw_fbo, self.read_fbo]);
            MLHeadTrackingDestroy(self.head_tracking);
            MLGraphicsDestroyClient(&mut self.graphics_client);
        }
    }
}

pub(crate) enum MagicLeapVRMessage {
    GetDisplayData(Sender<Result<VRDisplayData, MLResult>>),
    StartPresenting,
    StartFrame(f32, f32, VRResolveFrameData),
    StopFrame(VRLayer, PooledGLTextureId),
    StopPresenting,
}
