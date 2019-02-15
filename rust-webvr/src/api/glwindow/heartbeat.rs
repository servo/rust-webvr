use gleam::gl;
use gleam::gl::Gl;
use glutin::GlContext;
use glutin::GlWindow;
use rust_webvr_api::VRResolveFrameData;
use rust_webvr_api::VRMainThreadHeartbeat;
use std::rc::Rc;
use std::time::Duration;
use super::display::GlWindowVRDisplay;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

const TIMEOUT: Duration = Duration::from_millis(16);

pub struct GlWindowVRMainThreadHeartbeat {
    receiver: Receiver<GlWindowVRMessage>,
    gl_window: GlWindow,
    gl: Rc<dyn Gl>,
    presenting: bool,
    timestamp: f64,
    texture_id: gl::GLuint,
    framebuffer_id: gl::GLuint,
}

impl VRMainThreadHeartbeat for GlWindowVRMainThreadHeartbeat {
    fn heartbeat(&mut self) {
       debug!("VR heartbeat start");
       loop {
           // If we are presenting, we block the main thread on the VR thread.
           let msg = if self.presenting {
               self.receiver.recv_timeout(TIMEOUT).ok()
           } else {
               self.receiver.try_recv().ok()
           };

           match msg {
               Some(msg) => if self.handle_msg(msg) { break; },
               None => break,
           };
        }
        debug!("VR heartbeat stop");
    }

    fn heart_racing(&self) -> bool {
        self.presenting
    }
}

impl GlWindowVRMainThreadHeartbeat {
    pub(crate) fn new(
        receiver: Receiver<GlWindowVRMessage>, 
        gl_window: GlWindow,
        gl: Rc<Gl>,
    ) -> GlWindowVRMainThreadHeartbeat {
        debug!("Creating VR heartbeat");
        GlWindowVRMainThreadHeartbeat {
            receiver: receiver,
            gl_window: gl_window,
            gl: gl,
            presenting: false,
            timestamp: 0.0,
            texture_id: 0,
            framebuffer_id: 0,
        }
    }

    fn handle_msg(&mut self, msg: GlWindowVRMessage) -> bool {
           match msg {
               GlWindowVRMessage::StartPresenting => {
                    debug!("VR starting");
                    self.gl_window.show();
                    self.presenting = true;
                    true
               },
               GlWindowVRMessage::StartFrame(near, far, mut resolver) => {
                   debug!("VR start frame");
                   let timestamp = self.timestamp;
                   let size = self.gl_window.get_inner_size().expect("No window size");
                   let hidpi = self.gl_window.get_hidpi_factor();
                   let size = size.to_physical(hidpi);
                   let data = GlWindowVRDisplay::frame_data(timestamp, size, near, far);
                   let _ = resolver.resolve(data);
                   self.timestamp = self.timestamp + 1.0;
                   false
               },
               GlWindowVRMessage::StopFrame(width, height, buffer) => {
                   debug!("VR stop frame {}x{} ({})", width, height, buffer.len());
                   // TODO: render the buffer contents
                   if let Err(err) = unsafe { self.gl_window.make_current() } {
		       error!("VR Display failed to make window current ({:?})", err);
		       return true;
		   }
                   if self.texture_id == 0 {
                       self.texture_id = self.gl.gen_textures(1)[0];
                       debug!("Generated texture {}", self.texture_id);
                   }
                   if self.framebuffer_id == 0 {
                       self.framebuffer_id = self.gl.gen_framebuffers(1)[0];
                       debug!("Generated framebuffer {}", self.framebuffer_id);
                   }

                   self.gl.clear_color(0.2, 0.3, 0.3, 1.0);
                   self.gl.clear(gl::COLOR_BUFFER_BIT);

                   self.gl.bind_texture(gl::TEXTURE_2D, self.texture_id);
                   self.gl.tex_image_2d(
                       gl::TEXTURE_2D,
                       0,
                       gl::RGBA as gl::GLint,
                       width as gl::GLsizei,
                       height as gl::GLsizei,
                       0,
                       gl::RGBA,
                       gl::UNSIGNED_BYTE,
                       Some(&buffer[..]),
                   );
                   self.gl.bind_texture(gl::TEXTURE_2D, 0);

                   self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, self.framebuffer_id);
                   self.gl.framebuffer_texture_2d(
                       gl::READ_FRAMEBUFFER, 
                       gl::COLOR_ATTACHMENT0,
                       gl::TEXTURE_2D,
                       self.texture_id,
                       0
                   );
                   self.gl.viewport(
                       0, 0, width as gl::GLsizei, height as gl::GLsizei,
                   );
                   self.gl.blit_framebuffer(
                       0, 0, width as gl::GLsizei, height as gl::GLsizei,
                       0, 0, width as gl::GLsizei, height as gl::GLsizei,
                       gl::COLOR_BUFFER_BIT,
                       gl::NEAREST,
                   );
                   self.gl.bind_framebuffer(gl::READ_FRAMEBUFFER, 0);

                   let _ = self.gl_window.swap_buffers();

                   let err = self.gl.get_error();
                   if err != 0 {
                       error!("Test VR Display GL error {}.", err);
                   }

                   true
               },
               GlWindowVRMessage::StopPresenting => {
                    debug!("VR stopping");
                    self.gl_window.hide();
                    self.presenting = false;
                    true
               },
           }
    }
}

pub(crate) enum GlWindowVRMessage {
    StartPresenting,
    StartFrame(f64, f64, VRResolveFrameData),
    StopFrame(u32, u32, Arc<Vec<u8>>),
    StopPresenting,
}
