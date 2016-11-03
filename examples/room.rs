extern crate webvr;
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
use self::cgmath::*;
use glium::GlObject;
use glium::backend::glutin_backend::GlutinFacade;
use glium::index::PrimitiveType;
use glium::texture::SrgbTexture2d;
use std::f32::consts::PI;
use std::mem;
use std::path::Path;

use webvr::{VRServiceManager, VRLayer, VRFrameData};

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


implement_vertex!(Vertex, position, uv);

type Vec3 = Vector3<f32>;
type Mat4 = Matrix4<f32>;
type Tex = glium::texture::SrgbTexture2d;


const VERTEX_SHADER_FB: &'static str = r#"
    #version 140
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
    #version 140

    in vec3 position;
    in vec2 uv;
    out vec2 v_uv;

    uniform mat4 projection;
    uniform mat4 view;
    uniform mat4 model;

    void main() {
        v_uv = uv;
        gl_Position = projection * view * model * vec4(position, 1.0);
    }
"#;

const FRAGMENT_SHADER: &'static str = r#"
    #version 140

    uniform sampler2D sampler;

    in vec2 v_uv;
    out vec4 color;

    void main() {
        color = texture(sampler, v_uv);
    }
"#;

const FRAGMENT_SHADER2: &'static str = r#"
    #version 140

    uniform sampler2D sampler;

    in vec2 v_uv;
    out vec4 color;

    void main() {
        color = texture(sampler, v_uv);
    }
"#;


fn load_texture(ctx: &GlutinFacade, name: &'static str) -> SrgbTexture2d {
    let path = format!("examples/res/{}", name);
    let image = image::open(&Path::new(&path)).unwrap().to_rgba();
    let image_dimensions = image.dimensions();
    let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
    SrgbTexture2d::new(ctx, image).unwrap()
}


struct Mesh<'a> {
    pub vertex_buffer: glium::VertexBuffer<Vertex>,
    pub index_buffer: glium::IndexBuffer<u16>, 
    pub transform: Mat4,
    pub texture:&'a Tex
}

impl<'a> Mesh<'a> {
    fn new_plane(ctx: &GlutinFacade, tex:&'a Tex, size:[f32;2], pos:[f32;3], rot:[f32;3], scale:[f32;3]) -> Mesh<'a> {
        let dx = size[0] * 0.5;
        let dy = size[1] * 0.5;
        let buffer = glium::VertexBuffer::new(ctx, &[
                Vertex { position: [-1.0 * dx, -1.0 * dy, 0.0], uv: [0.0, 0.0] },
                Vertex { position: [-1.0 * dx,  1.0 * dy, 0.0], uv: [0.0, 1.0] },
                Vertex { position: [ 1.0 * dx,  1.0 * dy, 0.0], uv: [1.0, 1.0] },
                Vertex { position: [ 1.0 * dx, -1.0 * dy, 0.0], uv: [1.0, 0.0] }]).unwrap();
        let index_buffer = glium::IndexBuffer::new(ctx, PrimitiveType::TriangleStrip,
                                               &[1 as u16, 2, 0, 3]).unwrap();

        let rotation = Matrix4::from(Euler { x: Rad(rot[0]), y: Rad(rot[1]), z: Rad(rot[2]) });
        let scale = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2]);
        let translation = Matrix4::from_translation(Vec3::new(pos[0], pos[1], pos[2]));
        let matrix =  translation * scale * rotation;
    
        Mesh {
            vertex_buffer: buffer,
            index_buffer: index_buffer,
            transform: matrix,
            texture: tex
        }
    }

    #[allow(dead_code)]
    fn new_sphere(ctx: &GlutinFacade, tex:&'a Tex, radius: f32, pos:[f32;3], rot:[f32;3]) -> Mesh<'a> {

        let sw = 80; // width segments
        let sh = 60; // heieght segments

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
            vertex_buffer: glium::VertexBuffer::new(ctx, &vertices).unwrap(),
            index_buffer: glium::IndexBuffer::new(ctx, PrimitiveType::TriangleStrip, &indices).unwrap(),
            transform: matrix,
            texture: tex
        }
    }

    fn new_quad(ctx: &GlutinFacade, tex:&'a Tex) -> Mesh<'a> {
        let vertices = [
                Vertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0, 0.0], uv: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0, 0.0], uv: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0, 0.0], uv: [1.0, 0.0] }
        ];
        let indices = [1 as u16, 2, 0, 3];

        Mesh {
            vertex_buffer: glium::VertexBuffer::new(ctx, &vertices).unwrap(),
            index_buffer: glium::IndexBuffer::new(ctx, PrimitiveType::TriangleStrip, &indices).unwrap(),
            transform: Matrix4::<f32>::identity(),
            texture: tex
        }

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

fn matrix_to_uniform<'a>(matrix:&'a Mat4) -> &'a[[f32; 4]; 4] {
    unsafe { mem::transmute(matrix) }
}

fn vec_to_quaternion(raw: &[f32; 4]) -> Quaternion<f32> {
    Quaternion::new(raw[0], raw[1], raw[2], raw[3])
}

fn vec_to_translation(raw: &[f32; 3]) -> Mat4 {
    Mat4::from_translation(Vector3::new(raw[0], raw[1], raw[2]))
}

pub fn main() {
     // Initialize VR Services
    let mut vr = VRServiceManager::new();
    // Register default VRService implementations and initialize them.
    // Default VRServices are specified using cargo features.
    vr.register_defaults();
    // Add a mock service to allow running the demo when no VR Device is available.
    // If no VR service is found the demo fallbacks to the Mock.
    vr.register_mock();
    // Intialize all registered VR Services
    vr.initialize_services();

    // Get found VR devices
    let devices = vr.get_devices();

    if devices.len() > 0 {
        println!("Found {} VR devices: ", devices.len());
    } else { 
        println!("No VR devices found");
        return;
    }

    // Select first device
    let device = devices.get(0).unwrap();

    let device_data = device.borrow_mut().get_display_data();
    println!("VR Device data: {:?}", device_data);

    use glium::{DisplayBuild, Surface};

    let render_width = device_data.left_eye_parameters.render_width;
    let render_height = device_data.left_eye_parameters.render_height;
    let window_width = render_width;
    let window_height = (render_height as f32 * 0.5) as u32;

    let near = 0.1f64;
    let far = 1000.0f64;
    // Virtual room size
    let width = 5f32;
    let height = 3.0f32;
    let depth = 5.5f32;

    let ctx = glium::glutin::WindowBuilder::new().with_dimensions(window_width, window_height)
                                                 .build_glium().unwrap();

    println!("Loading textures...");
    let c = load_texture(&ctx, "floor.jpg");
    let floor_tex = load_texture(&ctx, "floor.jpg");
    let wall_tex = load_texture(&ctx, "wall.jpg");
    let sky_tex = load_texture(&ctx, "sky.jpg");
    println!("Textures loaded!");

    // texture to be used as a framebuffer
    let target_texture = glium::texture::Texture2d::empty(&ctx, render_width * 2, render_height).unwrap();
    let prog = glium::Program::from_source(&ctx, VERTEX_SHADER_MVP, FRAGMENT_SHADER, None).unwrap();
    let prog_fb = glium::Program::from_source(&ctx, VERTEX_SHADER_FB, FRAGMENT_SHADER2, None).unwrap();

    let mut meshes = Vec::new();
    // Sky sphere
    meshes.push(Mesh::new_sphere(&ctx, &sky_tex, 50.0, [0.0, 0.0, 0.0], [0.0, PI, 0.0]));
    // floor
    meshes.push(Mesh::new_plane(&ctx, &floor_tex, [width,depth], [0.0, 0.0, 0.0], [-PI * 0.5, 0.0, 0.0], [1.0,1.0,1.0]));
    // walls
    meshes.push(Mesh::new_plane(&ctx, &wall_tex, [width,height], [0.0, height*0.5, -depth * 0.5], [0.0, 0.0, 0.0], [1.0,1.0,1.0]));
    meshes.push(Mesh::new_plane(&ctx, &wall_tex, [width,height], [0.0, height*0.5, depth*0.5], [0.0, 0.0, 0.0], [-1.0,1.0,1.0]));
    meshes.push(Mesh::new_plane(&ctx, &wall_tex, [depth,height], [width*0.5, height*0.5, 0.0], [0.0, PI * 0.5, 0.0], [-1.0,1.0,1.0]));
    meshes.push(Mesh::new_plane(&ctx, &wall_tex, [depth,height], [-width*0.5, height*0.5, 0.0], [0.0, -PI * 0.5, 0.0], [-1.0,1.0,1.0]));

    // Fake texture to force glutin clean
    meshes.push(Mesh::new_plane(&ctx, &c, [0.0001,0.0001], [-width*0.5, height*0.5, 0.0], [0.0, -PI * 0.5, 0.0], [-1.0,1.0,1.0]));

    let fbo_to_screen = Mesh::new_quad(&ctx, &floor_tex);

    let mut render_params = glium::DrawParameters {
        .. Default::default()
    };

    let left_viewport = glium::Rect {
        left: 0,
        bottom: 0,
        width: render_width,
        height: render_height
    };

    let right_viewport = glium::Rect {
        left: render_width,
        bottom: 0,
        width: render_width,
        height: render_height
    };

    let mut standing_transform = if let Some(ref stage) = device_data.stage_parameters {
        vec_to_matrix(&stage.sitting_to_standing_transform).inverse_transform().unwrap()
    } else {
        // Stage parameters not avaialbe yet or unsupported
        // Assume human height 1.75m
        // vec_to_translation(&[0.0, 1.75, 0.0]).inverse_transform().unwrap()
        Matrix4::<f32>::identity()
    };

    let mut framebuffer = target_texture.as_surface();

    loop {
        device.borrow_mut().sync_poses();


        let device_data = device.borrow().get_display_data();
        if let Some(ref stage) = device_data.stage_parameters {
            // TODO: use event queue instead of checking this every frame
            standing_transform = vec_to_matrix(&stage.sitting_to_standing_transform).inverse_transform().unwrap();
        }

        let mut target = ctx.draw();
        framebuffer.clear_color(0.0, 0.0, 1.0, 1.0);

        let data: VRFrameData = device.borrow().get_frame_data(near, far);

        // Calculate view transform based on pose data
        // We can also use data.left_view_matrix instead, we use Pose for testing purposes
        let quaternion = data.pose.orientation.unwrap_or([0.0, 0.0, 0.0, 1.0]);
        let rotation_transform = Mat4::from(vec_to_quaternion(&quaternion));
        let position_transform = match data.pose.position {
            Some(ref position) => vec_to_translation(&position).inverse_transform().unwrap(),
            None => Matrix4::<f32>::identity()
        };
        
        let view = rotation_transform * position_transform * standing_transform;
        let left_eye_to_head = vec_to_translation(&device_data.left_eye_parameters.offset).inverse_transform().unwrap();
        let right_eye_to_head = vec_to_translation(&device_data.right_eye_parameters.offset).inverse_transform().unwrap();

        // render per eye to the FBO
        let eyes =  [
            (&left_viewport, &data.left_projection_matrix, &left_eye_to_head),
            (&right_viewport, &data.right_projection_matrix, &right_eye_to_head)
        ];

        for eye in &eyes {
            render_params.viewport = Some(*eye.0);
            let eye_view = view * eye.2;
            let projection = vec_to_matrix(eye.1);

            for mesh in &meshes {
                let uniforms = uniform! {
                    projection: *matrix_to_uniform(&projection),
                    view: *matrix_to_uniform(&eye_view),
                    model: *matrix_to_uniform(&mesh.transform),
                    sampler: mesh.texture
                };
               framebuffer.draw(&mesh.vertex_buffer, &mesh.index_buffer, &prog, &uniforms, &render_params).unwrap();
            }
        }

        // Render to HMD
        let layer = VRLayer {
            texture_id: target_texture.get_id(),
            .. Default::default()
        };
        device.borrow_mut().submit_frame(&layer);

        // render per eye to the FBO
        target.clear_color(1.0, 0.0, 0.0, 1.0);
        // render to desktop display
        let uniforms = uniform! {
            matrix: *matrix_to_uniform(&fbo_to_screen.transform),
            sampler: &target_texture
        };
        target.draw(&fbo_to_screen.vertex_buffer, &fbo_to_screen.index_buffer, &prog_fb, &uniforms, &Default::default()).unwrap();


        target.finish().unwrap();


        assert_no_gl_error!(ctx);

        for event in ctx.poll_events() {
            match event {
                glium::glutin::Event::Closed => return,
                _ => {}
            }
        }
    }
}