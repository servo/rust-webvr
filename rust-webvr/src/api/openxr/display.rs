use {VRDisplay, VRDisplayData, VRFrameData, VRFramebuffer, VRGamepadPtr, VRLayer};
use openxr::{Session, FormFactor, FrameStream, Instance, ViewConfigurationType};
use openxr::opengl::{OpenGL, SessionCreateInfo};
use openxr::sys::platform::{HDC, HGLRC};

pub struct OpenXrDisplay {
    session: Session<OpenGL>,
    frame_stream: FrameStream<OpenGL>,
}

impl OpenXrDisplay {
    pub fn new(instance: &Instance, h_dc: HDC, h_glrc: HGLRC) -> Result<OpenXrDisplay, String> {
        let system = instance
            .system(FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(|e| format!("{:?}", e))?;

        let (session, frame_stream) = unsafe {
            instance
                .create_session::<OpenGL>(
                    system,
                    &SessionCreateInfo::Windows {
                        h_dc,
                        h_glrc,
                    }
                )
                .map_err(|e| format!("{:?}", e))?
        };

        session
            .begin(ViewConfigurationType::PRIMARY_STEREO)
            .map_err(|e| format!("{:?}", e))?;

        let view_configuration_views = instance
            .enumerate_view_configuration_views(
                system, ViewConfigurationType::PRIMARY_STEREO
            )
            .map_err(|e| format!("{:?}", e))?;

        let _resolution = (
            view_configuration_views[0].recommended_image_rect_width,
            view_configuration_views[0].recommended_image_rect_height,
        );

        Ok(OpenXrDisplay {
            session,
            frame_stream,
        })
    }
}

impl VRDisplay for OpenXrDisplay {
    fn id(&self) -> u32 {
        unimplemented!()
    }

    fn data(&self) -> VRDisplayData {
        let capabilities = VRDisplayCapabilities {
            has_position: true,
            has_orientation: true,
            has_external_display: false,
            can_present: true,
            presented_by_browser: false,
            max_layers: 1,
        };
        VRDisplayData {
            display_id: utils::new_id(),
            display_name: "".to_owned(),
            connected: true,
            capabilities,
            stage_parameters: None,
            left_eye_parameters,
            right_eye_parameters,
        }
    }

    fn immediate_frame_data(&self, _near_z: f64, _far_z: f64) -> VRFrameData {
        unimplemented!()
    }

    fn synced_frame_data(&self, _near_z: f64, _far_z: f64) -> VRFrameData {
        unimplemented!()
    }

    fn reset_pose(&mut self) {
        unimplemented!()
    }

    fn sync_poses(&mut self) {
        unimplemented!()
    }

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        unimplemented!()
    }

    fn bind_framebuffer(&mut self, _eye_index: u32) {
        unimplemented!()
    }

    fn fetch_gamepads(&mut self) -> Result<Vec<VRGamepadPtr>,String> {
        unimplemented!()
    }

    fn render_layer(&mut self, _layer: &VRLayer) {
        unimplemented!()
    }

    fn submit_frame(&mut self) {
        unimplemented!()
    }

    fn stop_present(&mut self) {
        unimplemented!()
    }
}
