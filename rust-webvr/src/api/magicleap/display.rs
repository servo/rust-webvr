use gleam::gl;
use gleam::gl::Gl;
use gleam::gl::types::GLint;
use gleam::gl::types::GLuint;
use rust_webvr_api::VRDisplay;
use rust_webvr_api::VRDisplayData;
use rust_webvr_api::VRFrameData;
use rust_webvr_api::VRFutureFrameData;
use rust_webvr_api::VRFramebuffer;
use rust_webvr_api::VRFramebufferAttributes;
use rust_webvr_api::VRGamepadPtr;
use rust_webvr_api::VRLayer;
use std::cell::RefCell;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use super::heartbeat::MagicLeapVRMessage;

pub type MagicLeapVRDisplayPtr = Arc<RefCell<MagicLeapVRDisplay>>;

pub struct MagicLeapVRDisplay {
    display_data: VRDisplayData,
    sender: Sender<MagicLeapVRMessage>,
    fbos: Vec<GLuint>,
    texture_id_pool: ArcPool<GLuint>,
}

unsafe impl Sync for MagicLeapVRDisplay {}

impl Drop for MagicLeapVRDisplay {
    fn drop(&mut self) {
        self.stop_present();
    }
}

impl VRDisplay for MagicLeapVRDisplay {
    fn id(&self) -> u32 {
        self.display_data.display_id
    }

    fn data(&self) -> VRDisplayData {
        self.display_data.clone()
    }

    fn immediate_frame_data(&self, _near: f64, _far: f64) -> VRFrameData {
        VRFrameData::default()
    }

    fn synced_frame_data(&self, _near: f64, _far: f64) -> VRFrameData {
        unimplemented!()
    }

    fn reset_pose(&mut self) {
        unimplemented!()
    }

    fn sync_poses(&mut self) {
        unimplemented!()
    }

    fn future_frame_data(&mut self, near: f64, far: f64) -> VRFutureFrameData {
        let (resolver, result) = VRFutureFrameData::blocked();
        let _ = self.sender.send(MagicLeapVRMessage::StartFrame(near as f32, far as f32, resolver));
        result
    }

    fn bind_framebuffer(&mut self, _eye_index: u32) {
        unimplemented!()
    }

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        unimplemented!()
    }

    fn render_layer(&mut self, _layer: &VRLayer) {
        unreachable!()
    }

    fn submit_frame(&mut self) {
        unreachable!()
    }

    fn submit_layer(&mut self, gl: &Gl, layer: &VRLayer) {
        // So... why does this blit exist? Well...
        // submit_layer is called from the WebGL thread,
        // but we need to display the texture in the main thread,
        // so we send it to the main thread for display.
        // the WebGL thread *cannot block* waiting on the main thread,
        // since this might introduce deadlock
        // https://github.com/servo/servo/issues/22914
        // so we have to return immediately.
        // Unfortunately, this means the WebGL thread may
        // then update the texture, and so we end up with the main
        // thread displaying a texture that the WebGL thread is in
        // the middle of updating, which produces flickering.
        // This is the same issue as https://github.com/servo/servo/issues/21838.
        // The trick we use to avoid this is to use a pool of GL textures,
        // and send the main thread an element from the pool.
        // We send it as an `PooledGLTextureId`, which uses an `Arc` under the hood,
        // so when the main thread is no longer using the texture, it gets returned
        // to the pool. It might be nice to use the same trick for webrender,
        // but that probably involves changing the webrender API.
        let pooled_id = self.blit_texture(gl, layer);
        let texture_id = pooled_id.texture_id();
        let layer = VRLayer { texture_id, ..layer.clone() };
        let _ = self.sender.send(MagicLeapVRMessage::StopFrame(layer, pooled_id));
    }

    fn start_present(&mut self, _attributes: Option<VRFramebufferAttributes>) {
        let _ = self.sender.send(MagicLeapVRMessage::StartPresenting);
    }

    fn stop_present(&mut self) {
        let _ = self.sender.send(MagicLeapVRMessage::StopPresenting);
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>, String> {
        Ok(vec![])
    }
}

impl MagicLeapVRDisplay {
    pub(crate) fn new(
        display_data: VRDisplayData,
        sender: Sender<MagicLeapVRMessage>
    ) -> MagicLeapVRDisplay {
        info!("Creating VRDisplay");
        let fbos = Vec::new();
        let texture_id_pool = ArcPool::new();
        MagicLeapVRDisplay { display_data, sender, fbos, texture_id_pool }
    }

    fn blit_texture(&mut self, gl: &Gl, layer: &VRLayer) -> PooledGLTextureId {
        // Sigh, all this code just to copy a texture...

        // The dimensions of the texture
        let (texture_w, texture_h) = layer
            .texture_size
            .map(|(w, h)| (w as GLint, h as GLint))
            .unwrap_or((2560, 960));

        // Save the current FBO bindings
        let mut current_fbos = [0, 0];
        unsafe { gl.get_integer_v(gl::DRAW_FRAMEBUFFER_BINDING, &mut current_fbos[0..]) };
        unsafe { gl.get_integer_v(gl::READ_FRAMEBUFFER_BINDING, &mut current_fbos[1..]) };
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Set the FBOs to be ours
        if self.fbos.len() < 2 { self.fbos = gl.gen_framebuffers(2); }
        gl.bind_framebuffer(gl::READ_FRAMEBUFFER, self.fbos[0]);
        gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, self.fbos[1]);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Bind the source texture to the read FBO
        gl.framebuffer_texture_2d(gl::READ_FRAMEBUFFER,
                                  gl::COLOR_ATTACHMENT0,
                                  gl::TEXTURE_2D,
                                  layer.texture_id, 0);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Set the viewport
        gl.viewport(0, 0, texture_w, texture_h);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Get the destination texture from the pool, or create a new one
        let pooled_id = self.pooled_texture_id(gl, texture_w, texture_h);
        let texture_id = pooled_id.texture_id();

        // Bind the destination texture to the draw FBO
        gl.framebuffer_texture_2d(gl::DRAW_FRAMEBUFFER,
                                  gl::COLOR_ATTACHMENT0,
                                  gl::TEXTURE_2D,
                                  texture_id, 0);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Do the blit
        debug!("Blitting from {} to {} ({}x{})", layer.texture_id, texture_id, texture_w, texture_h);
        gl.blit_framebuffer(0, 0, texture_w, texture_h,
                            0, 0, texture_w, texture_h,
                            gl::COLOR_BUFFER_BIT, gl::LINEAR);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Restore the old framebuffers
        gl.bind_framebuffer(gl::DRAW_FRAMEBUFFER, current_fbos[0] as GLuint);
        gl.bind_framebuffer(gl::READ_FRAMEBUFFER, current_fbos[1] as GLuint);
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Flush commands so they're seen by the main thread
        gl.flush();
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // Done!
        pooled_id
    }

    fn pooled_texture_id(&mut self, gl: &Gl, width: GLint, height: GLint) -> PooledGLTextureId {
        let texture_id = self.texture_id_pool.remove().unwrap_or_else(|| {
            let texture_id = gl.gen_textures(1)[0];
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            gl.bind_texture(gl::TEXTURE_2D, texture_id);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            gl.tex_image_2d(gl::TEXTURE_2D,
                            0,
                            gl::RGBA as GLint,
                            width, height,
                            0,
                            gl::RGBA,
                            gl::UNSIGNED_BYTE,
                            None);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            gl.bind_texture(gl::TEXTURE_2D, 0);
            debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

            texture_id
        });
        PooledGLTextureId(self.texture_id_pool.add(texture_id))
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

// A pooled GLTextureId

pub struct PooledGLTextureId(Arc<GLuint>);

impl PooledGLTextureId {
    pub fn texture_id(&self) -> GLuint {
        *self.0
    }
}
