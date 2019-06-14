use euclid::Angle;
use euclid::RigidTransform3D;
use euclid::Rotation3D;
use euclid::Vector3D;
use gleam::gl;
use gleam::gl::Gl;
use glutin::{WindowedContext, NotCurrent};
use glutin::EventsLoop;
use glutin::Event;
use glutin::VirtualKeyCode;
use glutin::WindowEvent;
use rust_webvr_api::VRResolveFrameData;
use rust_webvr_api::VRMainThreadHeartbeat;
use std::rc::Rc;
use std::time::Duration;
use super::display::GlWindowVRDisplay;
use super::service::EventsLoopFactory;
use std::sync::Arc;
use std::sync::mpsc::Receiver;

const TIMEOUT: Duration = Duration::from_millis(16);
const DELTA: f32 = 0.05;
const ANGLE: Angle<f32> = Angle { radians: 0.1 };

pub struct GlWindowVRMainThreadHeartbeat {
    receiver: Receiver<GlWindowVRMessage>,
    gl_context: Option<WindowedContext<NotCurrent>>,
    events_loop_factory: EventsLoopFactory,
    events_loop: Option<EventsLoop>,
    gl: Rc<dyn Gl>,
    presenting: bool,
    timestamp: f64,
    texture_id: gl::GLuint,
    framebuffer_id: gl::GLuint,
    view: RigidTransform3D<f32>,
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
        gl_context: WindowedContext<NotCurrent>,
        events_loop_factory: EventsLoopFactory,
        gl: Rc<Gl>,
    ) -> GlWindowVRMainThreadHeartbeat {
        debug!("Creating VR heartbeat");
        GlWindowVRMainThreadHeartbeat {
            receiver: receiver,
            gl_context: Some(gl_context),
            events_loop_factory: events_loop_factory,
            events_loop: None,
            gl: gl,
            presenting: false,
            timestamp: 0.0,
            texture_id: 0,
            framebuffer_id: 0,
            view: RigidTransform3D::identity(),
        }
    }

    fn handle_msg(&mut self, msg: GlWindowVRMessage) -> bool {
           match msg {
               GlWindowVRMessage::StartPresenting => {
                    debug!("VR starting");
                    self.gl_context.as_ref().unwrap().window().show();
                    self.presenting = true;
		    if self.events_loop.is_none() {
                        self.events_loop = (self.events_loop_factory)().ok();
		    }
                    true
               },
               GlWindowVRMessage::StartFrame(near, far, mut resolver) => {
                   debug!("VR start frame");
                   self.handle_window_events();
                   let timestamp = self.timestamp;
                   let window = self.gl_context.as_ref().unwrap().window();
                   let size = window.get_inner_size().expect("No window size");
                   let hidpi = window.get_hidpi_factor();
                   let size = size.to_physical(hidpi);
                   let view = self.view;
                   let data = GlWindowVRDisplay::frame_data(timestamp, size, near, far, view);
                   let _ = resolver.resolve(data);
                   self.timestamp = self.timestamp + 1.0;
                   false
               },
               GlWindowVRMessage::StopFrame(width, height, buffer) => {
                   debug!("VR stop frame {}x{} ({})", width, height, buffer.len());
                   // TODO: render the buffer contents
                   let context = self.gl_context.take().expect("Context was current");
                   let context = match unsafe { context.make_current() } {
                       Err(err) => {
                           error!("VR Display failed to make window current ({:?})", err);
                           return true;
                       },
                       Ok(context) => context,
                   };
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

                   let _ = context.swap_buffers();

                   let err = self.gl.get_error();
                   if err != 0 {
                       error!("Test VR Display GL error {}.", err);
                   }

                   let context = match unsafe { context.make_not_current() } {
                       Err(err) => {
                           error!("VR Display failed to make window non current ({:?})", err);
                           return true;
                       },
                       Ok(context) => context,
                   };

                   self.gl_context = Some(context);

                   true
               },
               GlWindowVRMessage::StopPresenting => {
                    debug!("VR stopping");
                    self.gl_context.as_ref().unwrap().window().hide();
                    self.presenting = false;
                    true
               },
           }
    }

    fn handle_window_events(&mut self) {
        let view = &mut self.view;
        if let Some(ref mut events_loop) = self.events_loop {
            events_loop.poll_events(|event| {
                if let Event::WindowEvent { event: WindowEvent::KeyboardInput { input, .. }, .. } = event {
                    if let Some(key_code) = input.virtual_keycode {
                        let delta = match key_code {
                            VirtualKeyCode::Up => RigidTransform3D::from_translation(Vector3D::new(0.0, 0.0, DELTA)),
                            VirtualKeyCode::Down => RigidTransform3D::from_translation(Vector3D::new(0.0, 0.0, -DELTA)),
                            VirtualKeyCode::Left => RigidTransform3D::from_translation(Vector3D::new(-DELTA, 0.0, 0.0)),
                            VirtualKeyCode::Right => RigidTransform3D::from_translation(Vector3D::new(DELTA, 0.0, 0.0)),
                            VirtualKeyCode::W => RigidTransform3D::from_rotation(Rotation3D::around_x(ANGLE)),
                            VirtualKeyCode::S => RigidTransform3D::from_rotation(Rotation3D::around_x(-ANGLE)),
                            VirtualKeyCode::A => RigidTransform3D::from_rotation(Rotation3D::around_y(ANGLE)),
                            VirtualKeyCode::D => RigidTransform3D::from_rotation(Rotation3D::around_y(-ANGLE)),
                            _ => RigidTransform3D::identity(),
                        };
                        *view = view.post_mul(&delta);
                    }
                }
            })
        }
    }
}

pub(crate) enum GlWindowVRMessage {
    StartPresenting,
    StartFrame(f64, f64, VRResolveFrameData),
    StopFrame(u32, u32, Arc<Vec<u8>>),
    StopPresenting,
}
