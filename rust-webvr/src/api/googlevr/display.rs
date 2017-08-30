#![cfg(feature = "googlevr")]
use {VRDisplay, VRDisplayData, VRDisplayCapabilities, VRFramebuffer, VRFramebufferAttributes,
    VREvent, VRDisplayEvent, VREyeParameters, VRFrameData, VRLayer, VRViewport};
use super::service::GoogleVRService;
use rust_webvr_api::utils;
#[cfg(target_os="android")]
use rust_webvr_api::jni_utils::JNIScope;
use gl;
use gvr_sys as gvr;
use gvr_sys::gvr_feature::*;
use gvr_sys::gvr_color_format_type::*;
use gvr_sys::gvr_depth_stencil_format_type::*;
use std::ffi::CStr;
use std::sync::Arc;
use std::cell::RefCell;
use std::ptr;
use std::mem;
use std::sync::Mutex;

pub type GoogleVRDisplayPtr = Arc<RefCell<GoogleVRDisplay>>;

// 50ms is a good estimate recommended by the GVR Team.
// It takes in account the time between frame submission (without vsync) and 
// when the rendered image is sent to the physical pixels on the display.
const PREDICTION_OFFSET_NANOS: i64 = 50000000; // 50ms

pub struct GoogleVRDisplay {
    service: *const GoogleVRService,
    ctx: *mut gvr::gvr_context,
    viewport_list: *mut gvr::gvr_buffer_viewport_list,
    left_eye_vp: *mut gvr::gvr_buffer_viewport,
    right_eye_vp: *mut gvr::gvr_buffer_viewport,
    render_size: gvr::gvr_sizei,
    swap_chain: *mut gvr::gvr_swap_chain,
    frame: *mut gvr::gvr_frame,
    frame_bound: bool,
    synced_head_matrix: gvr::gvr_mat4f,
    fbo_id: u32,
    fbo_texture: u32,
    display_id: u32,
    presenting: bool,
    paused: bool,
    new_events_hint: bool,
    pending_events: Mutex<Vec<VREvent>>,
    processed_events: Mutex<Vec<VREvent>>,
    attributes: VRFramebufferAttributes,
}

unsafe impl Send for GoogleVRDisplay {}
unsafe impl Sync for GoogleVRDisplay {}

impl VRDisplay for GoogleVRDisplay {

    fn id(&self) -> u32 {
        self.display_id
    }

    fn data(&self) -> VRDisplayData {
        let mut data = VRDisplayData::default();

        let (vendor, model) = unsafe {
            (to_string(gvr::gvr_get_viewer_vendor(self.ctx)), to_string(gvr::gvr_get_viewer_model(self.ctx)))
        };
        if vendor.is_empty() {
            data.display_name = model;
        } else {
            data.display_name = format!("{} {}", vendor, model);
        }
        data.display_id = self.display_id;
        data.connected = true;
    
        self.fetch_capabilities(&mut data.capabilities);
        unsafe {
            self.fetch_eye_parameters(&mut data.left_eye_parameters, &mut data.right_eye_parameters);
        }
        data.stage_parameters = None;

        data
    }

    fn inmediate_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let mut data = VRFrameData::default();
        unsafe {
            let time = gvr::gvr_get_time_point_now();
            let head_matrix = self.fetch_head_matrix(&time);
            self.fetch_frame_data(&mut data, &head_matrix, near as f32, far as f32);
        };
        
        data
    }

    fn synced_frame_data(&self, near: f64, far: f64) -> VRFrameData {
        let mut data = VRFrameData::default();
        self.fetch_frame_data(&mut data, &self.synced_head_matrix, near as f32, far as f32);
        
        data
    }

    fn reset_pose(&mut self) {
        // On the Daydream platform, recentering is handled automatically and should never
        // be triggered programatically by applications. Hybrid apps that support both
        // Only call this function when in Cardboard mode
        unsafe {
            if gvr::gvr_get_viewer_type(self.ctx) == gvr::gvr_viewer_type::GVR_VIEWER_TYPE_CARDBOARD as i32 {
                gvr::gvr_reset_tracking(self.ctx);
            }
        }
    }

    fn sync_poses(&mut self) {
        self.handle_events();
        if !self.presenting {
            self.start_present(None);
        }

        unsafe {
            if !self.frame.is_null() {
                warn!("submit_frame not called");
                // Release acquired frame if the user has not called submit_Frame()
                gvr::gvr_frame_submit(mem::transmute(&self.frame), self.viewport_list, self.synced_head_matrix);
            }

            self.update_recommended_buffer_viewports();
            // Handle resize
            let size = self.recommended_render_size();
            if size.width != self.render_size.width || size.height != self.render_size.height {
                gvr::gvr_swap_chain_resize_buffer(self.swap_chain, 0, size);
                self.render_size = size;
            }

            self.frame = gvr::gvr_swap_chain_acquire_frame(self.swap_chain);
        }

        // Predict head matrix
        let mut time = unsafe { gvr::gvr_get_time_point_now() };
        time.monotonic_system_time_nanos += PREDICTION_OFFSET_NANOS;
        self.synced_head_matrix = self.fetch_head_matrix(&time);
    }

    fn bind_framebuffer(&mut self, _eye_index: u32) {
        // No op
        if self.frame.is_null() {
            warn!("null frame with context");
            return;
        }

        unsafe {
            if self.frame_bound {
                // Required to avoid some warnings from the GVR SDK.
                // It doesn't like binding the same framebuffer multiple times.
                gvr::gvr_frame_unbind(self.frame);
            }
            gvr::gvr_frame_bind_buffer(self.frame, 0);
            self.frame_bound = true;
        }
    }

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        if self.attributes.multiview {
            vec![VRFramebuffer {
                    id: 0,
                    attributes: self.attributes,
                    viewport: VRViewport::new(0, 0, self.render_size.width / 2, self.render_size.height)
                }]
        } else {
            vec![VRFramebuffer {
                    id: 0,
                    attributes: self.attributes,
                    viewport: VRViewport::new(0, 0, self.render_size.width / 2, self.render_size.height)
                },
                VRFramebuffer {
                    id: 1,
                    attributes: self.attributes,
                    viewport: VRViewport::new(self.render_size.width / 2, 0, self.render_size.width/2, self.render_size.height)
                }]
        }
    }

    fn render_layer(&mut self, layer: &VRLayer) {
        if self.frame.is_null() {
            warn!("null frame when calling render_layer");
            return;
        }
        debug_assert!(self.fbo_id > 0);

        unsafe {
            // Save current fbo to restore it when the frame is submitted.
            let mut current_fbo = 0;
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut current_fbo);

            if self.fbo_texture != layer.texture_id {
                // Attach external texture to the used later in BlitFramebuffer.
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.fbo_id);
                gl::FramebufferTexture2D(gl::FRAMEBUFFER,
                                        gl::COLOR_ATTACHMENT0,
                                        gl::TEXTURE_2D,
                                        layer.texture_id, 0);
                self.fbo_texture = layer.texture_id;
            }

            let texture_size = layer.texture_size.unwrap_or_else(|| {
                (self.render_size.width as u32, self.render_size.height as u32)
            });

            // BlitFramebuffer: external texture to gvr pixel buffer
            gvr::gvr_frame_bind_buffer(self.frame, 0);
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo_id);
            gl::BlitFramebuffer(0, 0, texture_size.0 as i32, texture_size.1 as i32,
                                0, 0, self.render_size.width, self.render_size.height,
                                gl::COLOR_BUFFER_BIT, gl::LINEAR);
            gvr::gvr_frame_unbind(self.frame);
            self.frame_bound = false;
            // Restore bound fbo
            gl::BindFramebuffer(gl::FRAMEBUFFER, current_fbo as u32);

            // set up uvs
            gvr::gvr_buffer_viewport_set_source_uv(self.left_eye_vp, gvr_texture_bounds(&layer.left_bounds));
            gvr::gvr_buffer_viewport_set_source_uv(self.right_eye_vp, gvr_texture_bounds(&layer.right_bounds));
        }
    }

    fn submit_frame(&mut self) {
        if self.frame.is_null() {
            warn!("null frame with context");
            return;
        }

        unsafe {
            if self.frame_bound {
                gvr::gvr_frame_unbind(self.frame);
                self.frame_bound = false;
            }
            // submit frame
            gvr::gvr_frame_submit(mem::transmute(&self.frame), self.viewport_list, self.synced_head_matrix);
        }
    }

    #[cfg(target_os = "android")]
    fn start_present(&mut self, attributes: Option<VRFramebufferAttributes>) {
        if self.presenting {
            return;
        }
        if let Some(attributes) = attributes {
            self.attributes = attributes;
        }
        self.presenting = true;
        unsafe {
            if let Ok(jni_scope) = JNIScope::attach() {
                let jni = jni_scope.jni();
                let env = jni_scope.env;
                let method = jni_scope.get_method((*self.service).java_class, "startPresent", "()V", false);
                (jni.CallVoidMethod)(env, (*self.service).java_object, method);
            }
        }

        if self.swap_chain.is_null() {
            unsafe {
                self.initialize_gl();
                debug_assert!(!self.swap_chain.is_null());
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    fn start_present(&mut self, attributes: Option<VRFramebufferAttributes>) {
        if self.presenting {
            return;
        }
        if let Some(attributes) = attributes {
            self.attributes = attributes;
        }
        self.presenting = true;
        if self.swap_chain.is_null() {
            unsafe {
                self.initialize_gl();
                debug_assert!(!self.swap_chain.is_null());
            }
        }
    }

    // Hint to indicate that we are going to stop sending frames to the device
    #[cfg(target_os = "android")]
    fn stop_present(&mut self) {
        if !self.presenting {
            return;
        }
        self.presenting = false;
        unsafe {
            if let Ok(jni_scope) = JNIScope::attach() {
                let jni = jni_scope.jni();
                let env = jni_scope.env;
                let method = jni_scope.get_method((*self.service).java_class, "stopPresent", "()V", false);
                (jni.CallVoidMethod)(env, (*self.service).java_object, method);
            }
        }
    }

    #[cfg(not(target_os = "android"))]
    fn stop_present(&mut self) {
        self.presenting = false;
    }
}

impl GoogleVRDisplay {
    pub unsafe fn new(service: *const GoogleVRService,
                      ctx: *mut gvr::gvr_context) -> Arc<RefCell<GoogleVRDisplay>> {
        let list = gvr::gvr_buffer_viewport_list_create(ctx);

        // gvr_refresh_viewer_profile must be called before getting recommended bufer viewports.
        gvr::gvr_refresh_viewer_profile(ctx);

        // Gets the recommended buffer viewport configuration, populating a previously
        // allocated gvr_buffer_viewport_list object. The updated values include the
        // per-eye recommended viewport and field of view for the target.
        gvr::gvr_get_recommended_buffer_viewports(ctx, list);

        // Create viewport buffers for both eyes.
        let left_eye_vp = gvr::gvr_buffer_viewport_create(ctx);
        gvr::gvr_buffer_viewport_list_get_item(list, gvr::gvr_eye::GVR_LEFT_EYE as usize, left_eye_vp);
        let right_eye_vp = gvr::gvr_buffer_viewport_create(ctx);
        gvr::gvr_buffer_viewport_list_get_item(list, gvr::gvr_eye::GVR_RIGHT_EYE as usize, right_eye_vp);

        Arc::new(RefCell::new(GoogleVRDisplay {
            service: service,
            ctx: ctx,
            viewport_list: list,
            left_eye_vp: left_eye_vp,
            right_eye_vp: right_eye_vp,
            render_size: gvr::gvr_sizei {
                width: 0,
                height: 0,
            },
            swap_chain: ptr::null_mut(),
            frame: ptr::null_mut(),
            frame_bound: false,
            synced_head_matrix: gvr_identity_matrix(),
            fbo_id: 0,
            fbo_texture: 0,
            display_id: utils::new_id(),
            presenting: false,
            paused: false,
            new_events_hint: false,
            pending_events: Mutex::new(Vec::new()),
            processed_events: Mutex::new(Vec::new()),
            attributes: Default::default(),
        }))
    }

    unsafe fn initialize_gl(&mut self) {
        // Note: In some scenarios gvr_initialize_gl crashes if gvr_refresh_viewer_profile call isn't called before.
        gvr::gvr_refresh_viewer_profile(self.ctx);
        // Initializes gvr necessary GL-related objects.
        gvr::gvr_initialize_gl(self.ctx);

        // GVR_FEATURE_MULTIVIEW must be checked after gvr_initialize_gl is called or the function will crash. 
        if self.attributes.multiview && !gvr::gvr_is_feature_supported(self.ctx, GVR_FEATURE_MULTIVIEW as i32) {
            self.attributes.multiview = false;
            warn!("Multiview not supported. Fallback to standar framebuffer.")
        }

        // Create a framebuffer required to attach and
        // blit the external texture into the main gvr pixel buffer.
        gl::GenFramebuffers(1, &mut self.fbo_id);

        // Initialize gvr swap chain
        let spec = gvr::gvr_buffer_spec_create(self.ctx);
        self.render_size = self.recommended_render_size();

        if self.attributes.multiview {
            // Multiview requires half size because the buffer is a texture array with 2 half width layers.
            gvr::gvr_buffer_spec_set_multiview_layers(spec, 2);
            gvr::gvr_buffer_spec_set_size(spec, gvr::gvr_sizei {
                width: self.render_size.width / 2,
                height: self.render_size.height
            });
        } else {
            gvr::gvr_buffer_spec_set_size(spec, self.render_size);
        }

        if self.attributes.multisampling {
            gvr::gvr_buffer_spec_set_samples(spec, 2);
        } else {
            gvr::gvr_buffer_spec_set_samples(spec, 0);
        }
        gvr::gvr_buffer_spec_set_color_format(spec, GVR_COLOR_FORMAT_RGBA_8888 as i32);

        if self.attributes.depth {
            gvr::gvr_buffer_spec_set_depth_stencil_format(spec, GVR_DEPTH_STENCIL_FORMAT_DEPTH_16 as i32);
        } else {
            gvr::gvr_buffer_spec_set_depth_stencil_format(spec, GVR_DEPTH_STENCIL_FORMAT_NONE as i32);
        }

        self.swap_chain = gvr::gvr_swap_chain_create(self.ctx, mem::transmute(&spec), 1);
        gvr::gvr_buffer_spec_destroy(mem::transmute(&spec));
    }

    fn fetch_capabilities(&self, capabilities: &mut VRDisplayCapabilities) {
        capabilities.can_present = true;
        capabilities.has_orientation = true;
        capabilities.has_external_display = false;
        capabilities.has_position = false;
    }

    unsafe fn fetch_eye(&self, out: &mut VREyeParameters, eye: gvr::gvr_eye, viewport: *mut gvr::gvr_buffer_viewport) {
        let eye_fov = gvr::gvr_buffer_viewport_get_source_fov(viewport);

        out.field_of_view.up_degrees = eye_fov.top as f64;
        out.field_of_view.right_degrees = eye_fov.right as f64;
        out.field_of_view.down_degrees = eye_fov.bottom as f64;
        out.field_of_view.left_degrees = eye_fov.left as f64;

        let eye_mat = gvr::gvr_get_eye_from_head_matrix(self.ctx, eye as i32);
        out.offset = [-eye_mat.m[0][3], -eye_mat.m[1][3], -eye_mat.m[2][3]];
    }

    fn recommended_render_size(&self) -> gvr::gvr_sizei {
        // GVR SDK states that thee maximum effective render target size can be very large.
        // Most applications need to scale down to compensate.
        // Half pixel sizes are used by scaling each dimension by sqrt(2)/2 ~= 7/10ths.
        let render_target_size = unsafe { gvr::gvr_get_maximum_effective_render_target_size(self.ctx) };
        gvr::gvr_sizei {
            width: (7 * render_target_size.width) / 10,
            height: (7 * render_target_size.height) / 10
        }
    }

    unsafe fn fetch_eye_parameters(&self, left: &mut VREyeParameters, right: &mut VREyeParameters) {
        // Set fov and offset
        self.fetch_eye(left, gvr::gvr_eye::GVR_LEFT_EYE, self.left_eye_vp);
        self.fetch_eye(right, gvr::gvr_eye::GVR_RIGHT_EYE, self.right_eye_vp);

        let render_size = self.recommended_render_size();
        
        left.render_width = render_size.width as u32 / 2;
        left.render_height = render_size.height as u32;
        right.render_width = left.render_width;
        right.render_height = left.render_height;
    }

    fn fetch_head_matrix(&self, next_vsync: &gvr::gvr_clock_time_point) -> gvr::gvr_mat4f {
        unsafe {
            let m = gvr::gvr_get_head_space_from_start_space_rotation(self.ctx, *next_vsync);
            gvr::gvr_apply_neck_model(self.ctx, m, 1.0)
        }
    }

    fn fetch_frame_data(&self,
                        out: &mut VRFrameData,
                        head_mat: &gvr::gvr_mat4f,
                        near: f32,
                        far: f32) {
    
        self.update_recommended_buffer_viewports();

        // Get matrices from gvr.
        let left_eye = unsafe { gvr::gvr_get_eye_from_head_matrix(self.ctx, gvr::gvr_eye::GVR_LEFT_EYE as i32) };
        let right_eye = unsafe { gvr::gvr_get_eye_from_head_matrix(self.ctx, gvr::gvr_eye::GVR_RIGHT_EYE as i32) };

        // Convert gvr matrices to rust slices.
        let head_matrix = gvr_mat4_to_array(&head_mat);
        let mut view_matrix:[f32; 16] = unsafe { mem::uninitialized() };
        utils::inverse_matrix(&head_matrix, &mut view_matrix);

        let left_eye = gvr_mat4_to_array(&left_eye);
        let right_eye = gvr_mat4_to_array(&right_eye);

        // View matrix must by multiplied by each eye_to_head transformation matrix.
        utils::multiply_matrix(&left_eye, &view_matrix, &mut out.left_view_matrix);
        utils::multiply_matrix(&right_eye, &view_matrix, &mut out.right_view_matrix);

        // Projection matrices
        let left_fov = unsafe { gvr::gvr_buffer_viewport_get_source_fov(self.left_eye_vp) };
        let right_fov = unsafe { gvr::gvr_buffer_viewport_get_source_fov(self.right_eye_vp) };
        out.left_projection_matrix = fov_to_projection_matrix(&left_fov, near, far);
        out.right_projection_matrix = fov_to_projection_matrix(&right_fov, near, far);

        out.pose.orientation = Some(utils::matrix_to_quat(&view_matrix));
        out.pose.position = Some([view_matrix[12], view_matrix[13], view_matrix[14]]);

        // Timestamp
        out.timestamp = utils::timestamp();
    }

    fn update_recommended_buffer_viewports(&self) {
        unsafe {
            gvr::gvr_get_recommended_buffer_viewports(self.ctx, self.viewport_list);
            if self.attributes.multiview {
                // gvr_get_recommended_buffer_viewports function assumes that the client is not 
                // using multiview to render to multiple layers simultaneously.
                // The uv and source layers need to be updated for multiview.
                let fullscreen_uv = gvr_texture_bounds(&[0.0, 0.0, 1.0, 1.0]);
                // Left eye
                gvr::gvr_buffer_viewport_set_source_uv(self.left_eye_vp, fullscreen_uv);
                gvr::gvr_buffer_viewport_set_source_layer(self.left_eye_vp, 0);
                // Right eye
                gvr::gvr_buffer_viewport_set_source_uv(self.right_eye_vp, fullscreen_uv);
                gvr::gvr_buffer_viewport_set_source_layer(self.right_eye_vp, 1);
                // Update viewport list
                gvr::gvr_buffer_viewport_list_set_item(self.viewport_list, 0, self.left_eye_vp);
                gvr::gvr_buffer_viewport_list_set_item(self.viewport_list, 1, self.right_eye_vp);
            }
        }
    }

    // Warning: this function is called from java Main thread
    // Use mutexes to ensure thread safety and process the event in sync with the render loop.
    #[allow(dead_code)]
    pub fn pause(&mut self) {
        let mut pending = self.pending_events.lock().unwrap();
        pending.push(VRDisplayEvent::Pause(self.display_id).into());

        self.new_events_hint = true;
    }

    // Warning: this function is called from java Main thread
    // Use mutexes to ensure thread safety and process the event in sync with the render loop.
    #[allow(dead_code)]
    pub fn resume(&mut self) {
        let mut pending = self.pending_events.lock().unwrap();
        pending.push(VRDisplayEvent::Resume(self.display_id).into());

        self.new_events_hint = true;
    }

    fn handle_events(&mut self) {
        if !self.new_events_hint {
            // Optimization to avoid mutex locks every frame
            // It doesn't matter if events are processed in the next loop iteration
            return;
        }
        
        let mut pending: Vec<VREvent> = {
            let mut pending_events = self.pending_events.lock().unwrap();
            self.new_events_hint = false;
            let res = (*pending_events).drain(..).collect();
            res
        };
        

        for event in &pending {
            match *event {
                VREvent::Display(ref ev) => {
                    self.handle_display_event(&ev);
                },
                _ => {}
            }
        }

        let mut processed = self.processed_events.lock().unwrap();
        processed.extend(pending.drain(..));
    }

    fn handle_display_event(&mut self, event: &VRDisplayEvent) {
        match *event {
            VRDisplayEvent::Pause(_) => {
                if self.paused {
                    return;
                }
                unsafe {
                    gvr::gvr_pause_tracking(self.ctx);
                }
                self.paused = true;
            },
            VRDisplayEvent::Resume(_) => {
                if !self.paused {
                    return;
                }
                unsafe {
                    gvr::gvr_resume_tracking(self.ctx);
                    // Very important to call refresh after a resume event.
                    // If not called GvrLayout java view shows a black screen
                    gvr::gvr_refresh_viewer_profile(self.ctx);
                }
                self.paused = false;
            }
            _ => {}
        }
    }

    pub fn poll_events(&mut self, out: &mut Vec<VREvent>) {
        self.handle_events();
        let mut processed = self.processed_events.lock().unwrap();
        out.extend(processed.drain(..));
    }
}

impl Drop for GoogleVRDisplay {
    fn drop(&mut self) {
        unsafe {
            if self.fbo_id > 0 {
                gl::DeleteFramebuffers(1, mem::transmute(&self.fbo_id));
            }
            if !self.swap_chain.is_null() {
                gvr::gvr_swap_chain_destroy(mem::transmute(&self.swap_chain));
            }

            gvr::gvr_buffer_viewport_destroy(mem::transmute(&self.left_eye_vp));
            gvr::gvr_buffer_viewport_destroy(mem::transmute(&self.right_eye_vp));
            gvr::gvr_buffer_viewport_list_destroy(mem::transmute(&self.viewport_list));
        }
    }
}

// Helper functions

#[inline]
fn gvr_mat4_to_array(matrix: &gvr::gvr_mat4f) -> [f32; 16] {
    [matrix.m[0][0], matrix.m[0][1], matrix.m[0][2], matrix.m[0][3],
     matrix.m[1][0], matrix.m[1][1], matrix.m[1][2], matrix.m[1][3],
     matrix.m[2][0], matrix.m[2][1], matrix.m[2][2], matrix.m[2][3],
     matrix.m[3][0], matrix.m[3][1], matrix.m[3][2], matrix.m[3][3]]
}

#[inline]
fn fov_to_projection_matrix(fov: &gvr::gvr_rectf, near: f32, far: f32) -> [f32; 16] {
    let left = -fov.left.to_radians().tan() * near;
    let right = fov.right.to_radians().tan() * near;
    let top = fov.top.to_radians().tan() * near;
    let bottom = -fov.bottom.to_radians().tan() * near;
    frustum_matrix(left, right, bottom, top, near, far)
}

fn frustum_matrix(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> [f32; 16] {
    let r_width  = 1.0 / (right - left);
    let r_height = 1.0 / (top - bottom);
    let r_depth  = 1.0 / (near - far);
    let x = 2.0 * (near * r_width);
    let y = 2.0 * (near * r_height);
    let a = (right + left) * r_width;
    let b = (top + bottom) * r_height;
    let c = (far + near) * r_depth;
    let d = 2.0 * (far * near * r_depth);

    [x, 0.0, 0.0, 0.0,
     0.0, y, 0.0, 0.0,
     a, b, c, -1.0,
     0.0, 0.0, d, 0.0]
}

#[inline]
fn gvr_identity_matrix() -> gvr::gvr_mat4f {
    gvr::gvr_mat4f {
        m: [[1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0]]
    }
}

#[inline]
fn gvr_texture_bounds(array: &[f32; 4]) -> gvr::gvr_rectf {
    gvr::gvr_rectf {
        left: array[0],
        right: array[0] + array[2],
        bottom: array[1],
        top: array[1] + array[3]
    }
}

fn to_string(ptr: *const ::std::os::raw::c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    let result = unsafe { CStr::from_ptr(ptr as *const _) };
    result.to_string_lossy().into()
}
