extern crate rust_webvr as webvr;
#[cfg(target_os = "android")]
extern crate android_injected_glue;
extern crate gleam;
use gleam::gl::{self, Gl, GLenum, GLint, GLuint};
extern crate glutin;
extern crate cgmath;
extern crate image;
use self::cgmath::*;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::mem;
use std::path::Path;
use std::{thread, time};

use webvr::{VRServiceManager, VREvent, VRDisplayEvent, VRLayer, VRFrameData};

type Vec3 = Vector3<f32>;
type Mat4 = Matrix4<f32>;

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
const SHADER_VERSION: &'static str = "#version 150\n";

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
const SHADER_VERSION: &'static str = "#version 300 es\n";

const VERTEX_SHADER_FB: &'static str = r#"
    uniform mat4 matrix;

    in vec3 position;
    in vec2 uv;
    out vec2 v_uv;

    void main() {
        v_uv = uv;
        gl_Position = matrix * vec4(position, 1.0);
    }
"#;

const VERTEX_SHADER_MVP: &'static str = r#"
    uniform mat4 projection;
    uniform mat4 view;
    uniform mat4 model;

    in vec3 position;
    in vec2 uv;
    out vec2 v_uv;

    void main() {
        v_uv = uv;
        gl_Position = projection * view * model * vec4(position, 1.0);
    }
"#;

const FRAGMENT_SHADER: &'static str = r#"
    precision highp float;
    uniform sampler2D sampler;
    in vec2 v_uv;
    out vec4 color;

    void main() {
        color = texture(sampler, v_uv);
    }
"#;

const FRAGMENT_SHADER2: &'static str = r#"
    precision highp float;
    uniform sampler2D sampler;

    in vec2 v_uv;
    out vec4 color;

    void main() {
        vec4 c = texture(sampler, v_uv);
        color = c;
        
    }
"#;

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    fn empty() -> Vertex {
        unsafe { mem::uninitialized() }
    }
}

fn prep_shader(shader: &str) -> String {
    format!("{}\n{}", SHADER_VERSION, shader)
}

fn build_shader(gl: &Gl, source: &str, shader_type: GLenum) -> GLuint {
    let source = prep_shader(source);
    let shader = gl.create_shader(shader_type);
    gl.shader_source(shader, &[source.as_str().as_bytes()]);
    gl.compile_shader(shader);
    let status = gl.get_shader_iv(shader, gl::COMPILE_STATUS);
    if status == 0 {
        let error = gl.get_shader_info_log(shader);
        panic!("Shader compilation failed. Error {:?} in shader {:?}", error, source);
    }

    shader
}
struct GLProgram {
    pub id: GLuint,
    pub locations: HashMap<&'static str, GLint>
}

impl GLProgram {
    fn loc(&self, uniform:&'static str) -> GLint {
        *self.locations.get(uniform).unwrap()
    }
}

fn build_program(gl: &Gl, vs: &str, fs: &str,
                 uniforms: &[&'static str],
                 attribs: &[&'static str]) -> GLProgram {
    let program = gl.create_program();
    assert!(program != 0);
    let vs =  build_shader(gl, vs, gl::VERTEX_SHADER);
    let fs = build_shader(gl, fs, gl::FRAGMENT_SHADER);
    gl.attach_shader(program, vs);
    gl.attach_shader(program, fs);
    let mut index = 0;
    for attrib in attribs {
        gl.bind_attrib_location(program, index, attrib);
        index += 1;
    }
    gl.link_program(program);
    let status = gl.get_program_iv(program, gl::LINK_STATUS);
    assert!(status != 0);

    let mut locations = HashMap::new();
    for uniform in uniforms {
        let loc = gl.get_uniform_location(program, uniform);
        assert!(loc != -1);
        locations.insert(*uniform, loc);
    }
    
    GLProgram {
        id: program,
        locations: locations
    }
}

fn build_vertex_buffer(gl: &Gl, vertices: &[Vertex]) -> GLuint {
    let buffer = gl.gen_buffers(1)[0];
    gl.bind_buffer(gl::ARRAY_BUFFER, buffer);
    gl::buffer_data(gl, gl::ARRAY_BUFFER, vertices, gl::STATIC_DRAW);

    buffer
}

fn build_indices_buffer(gl: &Gl, indices: &[u16]) -> GLuint {
    let buffer = gl.gen_buffers(1)[0];
    gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, buffer);
    gl::buffer_data(gl, gl::ELEMENT_ARRAY_BUFFER, indices, gl::STATIC_DRAW);

    buffer
}

#[cfg(target_os = "android")]
fn texture_path(name: &'static str) -> String {
    unsafe {
        let base = std::ffi::CStr::from_ptr((*android_injected_glue::get_app().activity).externalDataPath);
        let base = base.to_string_lossy();
        format!("{}/res/{}", base, name)
    }
}

#[cfg(not(target_os = "android"))]
fn texture_path(name: &'static str) -> String {
    format!("res/{}", name)
}

fn build_texture(gl: &Gl, name: &'static str) -> GLuint {
    let path = texture_path(name);
    let image = image::open(&Path::new(&path)).unwrap().to_rgba();
    let size = image.dimensions();
    let data = image.into_raw();
    // flip vertically required
    let data: Vec<u8> = data
                        .chunks(size.0 as usize * 4)
                        .rev()
                        .flat_map(|row| row.iter())
                        .map(|p| p.clone())
                        .collect();

    let texture = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D, texture);
    gl.tex_image_2d(gl::TEXTURE_2D, 0, gl::RGBA as i32, size.0 as i32, size.1 as i32,
                    0, gl::RGBA, gl::UNSIGNED_BYTE, Some(&data));
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

    assert!(gl.get_error() == gl::NO_ERROR);

    texture
}

fn build_fbo_texture(gl: &Gl, width: u32, height: u32) -> GLuint {
    let texture = gl.gen_textures(1)[0];
    gl.bind_texture(gl::TEXTURE_2D, texture);
    gl.tex_image_2d(gl::TEXTURE_2D, 0, gl::RGBA as i32, width as i32, height as i32,
                    0, gl::RGBA, gl::UNSIGNED_BYTE, None);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as GLint);
    gl.tex_parameter_i(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as GLint);

    assert_eq!(gl.get_error(), gl::NO_ERROR);

    texture
}

struct Mesh {
    pub vertex_buffer: GLuint,
    pub index_buffer: GLuint,
    pub index_count: u32,
    pub transform: Mat4,
    pub texture: GLuint,
}

impl Mesh {
    fn new_plane(gl: &Gl, tex: GLuint, size:[f32;2], pos:[f32;3], rot:[f32;3], scale:[f32;3]) -> Mesh {
        let dx = size[0] * 0.5;
        let dy = size[1] * 0.5;
        let buffer = build_vertex_buffer(gl, &[
                Vertex { position: [-1.0 * dx, -1.0 * dy, 0.0], uv: [0.0, 0.0] },
                Vertex { position: [-1.0 * dx,  1.0 * dy, 0.0], uv: [0.0, 1.0] },
                Vertex { position: [ 1.0 * dx,  1.0 * dy, 0.0], uv: [1.0, 1.0] },
                Vertex { position: [ 1.0 * dx, -1.0 * dy, 0.0], uv: [1.0, 0.0] }]);
        let index_buffer = build_indices_buffer(gl, &[1 as u16, 2, 0, 3]);

        let rotation = Matrix4::from(Euler { x: Rad(rot[0]), y: Rad(rot[1]), z: Rad(rot[2]) });
        let scale = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
        let translation = Matrix4::from_translation(Vec3::new(pos[0], pos[1], pos[2]));
        let matrix =  translation * scale * rotation;
    
        Mesh {
            vertex_buffer: buffer,
            index_buffer: index_buffer,
            index_count: 4,
            transform: matrix,
            texture: tex
        }
    }

    #[allow(dead_code)]
    fn new_sphere(gl: &Gl, tex: GLuint, radius: f32, pos:[f32;3], rot:[f32;3]) -> Mesh {
        let sw = 80u32; // width segments
        let sh = 60u32; // height segments

        let phi_start = 0.0;
        let phi_len = PI * 2.0;
        let theta_start = 0.0;
        let theta_len = PI;
        let theta_end = theta_start + theta_len;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for y in 0..sh + 1 {
            let v = y as f32 / sh as f32;
            for x in 0..sw + 1 {
                let u = x as f32/ sw as f32;

                let mut vertex = Vertex::empty();
                vertex.position = [
                    -radius * (phi_start + u * phi_len).cos() * (theta_start + v * theta_len).sin(),
                    radius * (phi_start + u * phi_len).sin() * (theta_start + v * theta_len).sin(),
                    radius * (theta_start + v * theta_len).cos(),
                ];
                vertex.uv = [u, 1.0 - v];

                vertices.push(vertex);

                // Add indices
                if y != 0 || theta_start > 0.0 {
                    indices.push((y * sw + x + 1) as u16);
                    indices.push((y * sw + x) as u16);
                    indices.push(((y + 1) * sw + x + 1) as u16);
                }
                if y != sh -1 || theta_end < PI {
                    indices.push((y * sw + x) as u16);
                    indices.push(((y + 1) * sw + x) as u16);
                    indices.push(((y + 1) * sw + x + 1) as u16);
                }
            }
        }


        let rotation = Matrix4::from(Euler { x: Rad(rot[0]), y: Rad(rot[1]), z: Rad(rot[2]) });
        let translation = Matrix4::from_translation(Vec3::new(pos[0], pos[1], pos[2]));
        let scale = Matrix4::from_nonuniform_scale(-1.0, 1.0, 1.0);
        let matrix = translation * scale * rotation;

        Mesh {
            vertex_buffer: build_vertex_buffer(gl, &vertices),
            index_buffer: build_indices_buffer(gl, &indices),
            index_count: indices.len() as u32,
            transform: matrix,
            texture: tex
        }
    }

    fn new_quad(gl: &Gl, tex: GLuint) -> Mesh {
        let vertices = [
                Vertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0, 0.0], uv: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0, 0.0], uv: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0, 0.0], uv: [1.0, 0.0] }
        ];
        let indices = [1 as u16, 2, 0, 3];

        Mesh {
            vertex_buffer: build_vertex_buffer(gl, &vertices),
            index_buffer: build_indices_buffer(gl, &indices),
            index_count: indices.len() as u32,
            transform: Matrix4::<f32>::identity(),
            texture: tex
        }
    }

    fn draw(&self, gl: &Gl) {
        gl.bind_texture(gl::TEXTURE_2D, self.texture);
        gl.bind_buffer(gl::ARRAY_BUFFER, self.vertex_buffer);
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
        gl.vertex_attrib_pointer(0, 3, gl::FLOAT, false, mem::size_of::<Vertex>() as i32, 0);
        gl.vertex_attrib_pointer(1, 2, gl::FLOAT, false, mem::size_of::<Vertex>() as i32, mem::size_of::<f32>() as u32 * 3);
        gl.draw_elements(gl::TRIANGLE_STRIP, self.index_count as i32, gl::UNSIGNED_SHORT, 0);
    }
}

// Helper utilities
fn vec_to_matrix<'a>(raw:&'a [f32; 16]) -> &'a Mat4 {
    unsafe { mem::transmute(raw) }
}

#[allow(dead_code)]
fn vec2_to_matrix<'a>(raw:&'a [[f32; 4]; 4]) -> &'a Mat4 {
    unsafe { mem::transmute(raw) }
}

#[allow(dead_code)]
fn vec_to_uniform<'a>(matrix: &'a [f32; 16]) -> &'a[[f32; 4]; 4] {
    unsafe { mem::transmute(matrix) }
}

fn matrix_to_uniform<'a>(matrix:&'a Mat4) -> &'a[f32; 16] {
    unsafe { mem::transmute(matrix) }
}

fn vec_to_quaternion(raw: &[f32; 4]) -> Quaternion<f32> {
    Quaternion::new(raw[3], raw[0], raw[1], raw[2])
}

fn vec_to_translation(raw: &[f32; 3]) -> Mat4 {
    Mat4::from_translation(Vector3::new(raw[0], raw[1], raw[2]))
}

#[cfg(not(any(target_arch = "arm", target_arch = "aarch64")))]
fn gl_version() -> glutin::GlRequest {
    glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 2))
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
fn gl_version() -> glutin::GlRequest {
    glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 0))
}

pub fn main() {
     // Initialize VR Services
    let mut vr = VRServiceManager::new();
    // Register default VRService implementations and initialize them.
    // Default VRServices are specified using cargo features.
    vr.register_defaults();
    // Add a mock service to allow running the demo when no VRDisplay is available.
    // If no VR service is found the demo fallbacks to the Mock.
    vr.register_mock();
    // Intialize all registered VR Services
    vr.initialize_services();

    // Get found VRDisplays
    let displays = vr.get_displays();

    if displays.len() > 0 {
        println!("Found {} VRDisplays: ", displays.len());
    } else { 
        println!("No VRDisplays found");
        return;
    }

    // Select first display
    let display = displays.get(0).unwrap();

    let display_data = display.borrow().data();
    println!("VRDisplay: {:?}", display_data);

    let render_width = display_data.left_eye_parameters.render_width;
    let render_height = display_data.left_eye_parameters.render_height;
    let window_width = render_width;
    let window_height = (render_height as f32 * 0.5) as u32;

    let near = 0.1f64;
    let far = 150.0f64;
    // Virtual room size
    let width = 5f32;
    let height = 3.0f32;
    let depth = 5.5f32;

    let window = glutin::WindowBuilder::new().with_dimensions(window_width, window_height) //.with_vsync()
                                             .with_gl(gl_version())
                                             .build().unwrap();
    unsafe {
        window.make_current().unwrap();
    }
    let gl = match gleam::gl::GlType::default() {
        gleam::gl::GlType::Gl => unsafe { gleam::gl::GlFns::load_with(|s| window.get_proc_address(s) as *const _) },
        gleam::gl::GlType::Gles => unsafe { gleam::gl::GlesFns::load_with(|s| window.get_proc_address(s) as *const _) },
    };
    let gl = &*gl;


    let screen_fbo = gl.get_integer_v(gl::FRAMEBUFFER_BINDING) as u32;
    let screen_size = window.get_inner_size_pixels().unwrap();
    let vao = gl.gen_vertex_arrays(1)[0];
    gl.bind_vertex_array(vao);
    gl.disable(gl::SCISSOR_TEST);
    gl.disable(gl::DEPTH_TEST);
    gl.disable(gl::STENCIL_TEST);

    println!("Loading textures...");
    let floor_tex = build_texture(gl, "floor.jpg");
    let wall_tex = build_texture(gl, "wall.jpg");
    let sky_tex = build_texture(gl, "sky.jpg");
    println!("Textures loaded!");

    // texture to be used as a framebuffer
    let target_texture = build_fbo_texture(gl, render_width * 2, render_height);
    let prog = build_program(gl, VERTEX_SHADER_MVP, FRAGMENT_SHADER,
                             &["projection", "view", "model", "sampler"], &["position", "uv"]);
    let prog_fb = build_program(gl, VERTEX_SHADER_FB, FRAGMENT_SHADER2,
                             &["matrix", "sampler"], &["position", "uv"]);

    let mut meshes = Vec::new();
    // Sky sphere
    meshes.push(Mesh::new_sphere(gl, sky_tex, 50.0, [0.0, 0.0, 0.0], [0.0, PI, 0.0]));
    // floor
    meshes.push(Mesh::new_plane(gl, floor_tex, [width,depth], [0.0, 0.0, 0.0], [-PI * 0.5, 0.0, 0.0], [1.0,1.0,1.0]));
    // walls
    meshes.push(Mesh::new_plane(gl, wall_tex, [width,height], [0.0, height*0.5, -depth * 0.5], [0.0, 0.0, 0.0], [1.0,1.0,1.0]));
    meshes.push(Mesh::new_plane(gl, wall_tex, [width,height], [0.0, height*0.5, depth*0.5], [0.0, 0.0, 0.0], [-1.0,1.0,1.0]));
    meshes.push(Mesh::new_plane(gl, wall_tex, [depth,height], [width*0.5, height*0.5, 0.0], [0.0, PI * 0.5, 0.0], [-1.0,1.0,1.0]));
    meshes.push(Mesh::new_plane(gl, wall_tex, [depth,height], [-width*0.5, height*0.5, 0.0], [0.0, -PI * 0.5, 0.0], [-1.0,1.0,1.0]));

    let fbo_to_screen = Mesh::new_quad(gl, target_texture);

    let left_viewport = (0i32, 0i32, render_width as i32, render_height as i32);
    let right_viewport = (render_width as i32, 0i32, render_width as i32, render_height as i32);

    let mut standing_transform = if let Some(ref stage) = display_data.stage_parameters {
        vec_to_matrix(&stage.sitting_to_standing_transform).inverse_transform().unwrap()
    } else {
        // Stage parameters not available yet or unsupported
        // Assume 0.75m transform height
        vec_to_translation(&[0.0, 1.75, 0.0]).inverse_transform().unwrap()
    };

    let framebuffer = gl.gen_framebuffers(1)[0];
    let depth_buffer = gl.gen_renderbuffers(1)[0];
    gl.bind_renderbuffer(gl::RENDERBUFFER, depth_buffer);
    gl.renderbuffer_storage(gl::RENDERBUFFER, gl::DEPTH_COMPONENT24, render_width as i32 * 2, render_height as i32);
    gl.bind_renderbuffer(gl::RENDERBUFFER, 0);
    gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);
    gl.framebuffer_texture_2d(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, target_texture, 0);
    gl.framebuffer_renderbuffer(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT,gl::RENDERBUFFER, depth_buffer);
    assert_eq!(gl.check_frame_buffer_status(gl::FRAMEBUFFER), gl::FRAMEBUFFER_COMPLETE);
    gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
    gl.bind_framebuffer(gl::FRAMEBUFFER, 0);

    let mut event_counter = 0u64;

    // We can use data.left_view_matrix or data.pose to render the scene
    let test_pose = false; 

    loop {
        display.borrow_mut().sync_poses();

        let display_data = display.borrow().data();
        if let Some(ref stage) = display_data.stage_parameters {
            // TODO: use event queue instead of checking this every frame
            standing_transform = vec_to_matrix(&stage.sitting_to_standing_transform).inverse_transform().unwrap();
        }

        gl.bind_framebuffer(gl::FRAMEBUFFER, framebuffer);
        gl.clear_color(1.0, 0.0, 0.0, 1.0);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

        let data: VRFrameData = display.borrow().synced_frame_data(near, far);

        let (left_view_matrix, right_view_matrix) = if test_pose {
             // Calculate view transform based on pose data
            let quaternion = data.pose.orientation.unwrap_or([0.0, 0.0, 0.0, 1.0]);
            let rotation_transform = Mat4::from(vec_to_quaternion(&quaternion));
            let position_transform = match data.pose.position {
                Some(ref position) => vec_to_translation(&position).inverse_transform().unwrap(),
                None => Matrix4::<f32>::identity()
            };
            let view = (rotation_transform * position_transform).inverse_transform().unwrap();
            let left_eye_to_head = vec_to_translation(&display_data.left_eye_parameters.offset);
            let right_eye_to_head = vec_to_translation(&display_data.right_eye_parameters.offset);
            ((view * left_eye_to_head).inverse_transform().unwrap(),
             (view * right_eye_to_head).inverse_transform().unwrap())
            
        } else {
            (*vec_to_matrix(&data.left_view_matrix), *vec_to_matrix(&data.right_view_matrix))
        };

        // render per eye to the FBO
        let eyes =  [
            (&left_viewport, &data.left_projection_matrix, &left_view_matrix),
            (&right_viewport, &data.right_projection_matrix, &right_view_matrix)
        ];

        gl.use_program(prog.id);
        gl.active_texture(gl::TEXTURE0);
        gl.uniform_1i(prog.loc("sampler"), 0);
        gl.enable_vertex_attrib_array(0); // position
        gl.enable_vertex_attrib_array(1); // uv

        for eye in &eyes {
            let viewport = eye.0;
            let projection = vec_to_matrix(eye.1);
            let eye_view = eye.2 * standing_transform;

            gl.uniform_matrix_4fv(prog.loc("projection"), false, matrix_to_uniform(&projection));
            gl.uniform_matrix_4fv(prog.loc("view"), false, matrix_to_uniform(&eye_view));
            gl.viewport(viewport.0, viewport.1, viewport.2, viewport.3);
            gl.scissor(viewport.0, viewport.1, viewport.2, viewport.3);

            for mesh in &meshes {
                gl.uniform_matrix_4fv(prog.loc("model"), false, matrix_to_uniform(&mesh.transform));
                mesh.draw(gl);
            }
        }

        gl.flush();

        // Render to HMD
        let layer = VRLayer {
            texture_id: target_texture,
            .. Default::default()
        };
        display.borrow_mut().submit_frame(&layer);

        // render to desktop display
        gl.bind_framebuffer(gl::FRAMEBUFFER, screen_fbo);
        gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);
        gl.use_program(prog_fb.id);
        gl.viewport(0, 0, screen_size.0 as i32, screen_size.1 as i32);
        gl.scissor(0, 0, screen_size.0 as i32, screen_size.1 as i32);
        gl.uniform_matrix_4fv(prog_fb.loc("matrix"), false, matrix_to_uniform(&fbo_to_screen.transform));
        gl.active_texture(gl::TEXTURE0);
        gl.uniform_1i(prog_fb.loc("sampler"), 0);
        fbo_to_screen.draw(gl);
        
        debug_assert_eq!(gl.get_error(), gl::NO_ERROR);

        // We don't need to swap buffer on Android because Daydream view is on top of the window.
        if !cfg!(target_os = "android") {
            match window.swap_buffers() {
                Err(error) => {
                    match error {
                        glutin::ContextError::ContextLost => {},
                        _ => { panic!("swap_buffers error: {:?}", error); },
                    }
                },
                Ok(_) => {},
            }
        }

        // debug controllers
        let gamepads = vr.get_gamepads();
        for gamepad in gamepads {
            let gamepad = gamepad.borrow();
            println!("Gamepad Data: {:?}", gamepad.data());
            println!("Gamepad State: {:?}", gamepad.state());
        }

        // We don't need to poll VR headset events every frame
        event_counter += 1;
        if event_counter % 100 == 0 {
            let mut paused = false;
            loop {
                for event in vr.poll_events() {
                    println!("VR Event: {:?}", event);
                    match event {
                        VREvent::Display(ev) => {
                            match ev {
                                VRDisplayEvent::Resume(..) => { paused = false;},
                                VRDisplayEvent::Pause(..) => { paused = true; },
                                _ => {},
                            }
                        },
                        _ => {}
                    }
                }
                if !paused {
                    break;
                }
                // Wait until Resume Event is received
                thread::sleep(time::Duration::from_millis(5));
            }
        }

        // Window Events
        for event in window.poll_events() {
            match event {
                glutin::Event::Closed => return,
                _ => {}
            }
        }
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
#[inline(never)]
#[allow(non_snake_case)]
pub extern "C" fn android_main(app: *mut ()) {
    android_injected_glue::android_main2(app as *mut _, move |_, _| main());
}
