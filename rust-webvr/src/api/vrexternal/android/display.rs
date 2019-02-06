use super::{mozgfx, VRExternalShmemPtr};
use rust_webvr_api::utils;
use std::cell::RefCell;
use std::mem;
use std::sync::Arc;
use {
    VRDisplay, VRDisplayData, VRDisplayEvent, VRFrameData, VRFramebuffer,
    VRFramebufferAttributes, VRLayer, VRViewport,
};

pub type VRExternalDisplayPtr = Arc<RefCell<VRExternalDisplay>>;

pub struct VRExternalDisplay {
    system_state: mozgfx::VRSystemState,
    browser_state: mozgfx::VRBrowserState,
    rendered_layer: Option<VRLayer>,
    shmem: VRExternalShmemPtr,
    display_id: u32,
    attributes: VRFramebufferAttributes,
    presenting: bool,
    events: Vec<VRDisplayEvent>,
}

impl VRExternalDisplay {
    pub fn new(shmem: VRExternalShmemPtr) -> VRExternalDisplayPtr {
        let system_state = shmem.as_mut().pull_system(&|_| true);
        let browser_state = shmem.as_mut().pull_browser();
        Arc::new(RefCell::new(VRExternalDisplay {
            system_state,
            browser_state,
            rendered_layer: None,
            shmem,
            display_id: utils::new_id(),
            attributes: Default::default(),
            presenting: false,
            events: Vec::new(),
        }))
    }

    pub fn poll_events(&mut self) -> Vec<VRDisplayEvent> {
        mem::replace(&mut self.events, Vec::new())
    }
}

impl VRExternalDisplay {
    fn push_browser(&mut self) {
        self.shmem.as_mut().push_browser(self.browser_state.clone());
    }
}

impl VRDisplay for VRExternalDisplay {
    fn id(&self) -> u32 {
        self.display_id
    }

    fn data(&self) -> VRDisplayData {
        let mut data = VRDisplayData::default();

        let state: &mozgfx::VRDisplayState = &self.system_state.displayState;
        data.display_name = state.mDisplayName.iter().map(|x| *x as char).collect();
        data.display_id = self.display_id;
        data.connected = state.mIsConnected;

        let flags = state.mCapabilityFlags;
        data.capabilities.has_position =
            (flags & mozgfx::VRDisplayCapabilityFlags_Cap_Position) != 0;
        data.capabilities.can_present = (flags & mozgfx::VRDisplayCapabilityFlags_Cap_Present) != 0;
        data.capabilities.has_orientation =
            (flags & mozgfx::VRDisplayCapabilityFlags_Cap_Orientation) != 0;
        data.capabilities.has_external_display =
            (flags & mozgfx::VRDisplayCapabilityFlags_Cap_External) != 0;

        data.stage_parameters = None;

        data.left_eye_parameters.offset = [
            state.mEyeTranslation[0].x,
            state.mEyeTranslation[0].y,
            state.mEyeTranslation[0].z,
        ];

        data.left_eye_parameters.render_width = state.mEyeResolution.width as u32;
        data.left_eye_parameters.render_height = state.mEyeResolution.height as u32;

        data.right_eye_parameters.offset = [
            state.mEyeTranslation[1].x,
            state.mEyeTranslation[1].y,
            state.mEyeTranslation[1].z,
        ];

        data.right_eye_parameters.render_width = state.mEyeResolution.width as u32;
        data.right_eye_parameters.render_height = state.mEyeResolution.height as u32;

        let l_fov = state.mEyeFOV[mozgfx::VRDisplayState_Eye_Eye_Left as usize];
        let r_fov = state.mEyeFOV[mozgfx::VRDisplayState_Eye_Eye_Right as usize];

        data.left_eye_parameters.field_of_view.up_degrees = l_fov.upDegrees;
        data.left_eye_parameters.field_of_view.right_degrees = l_fov.rightDegrees;
        data.left_eye_parameters.field_of_view.down_degrees = l_fov.downDegrees;
        data.left_eye_parameters.field_of_view.left_degrees = l_fov.leftDegrees;

        data.right_eye_parameters.field_of_view.up_degrees = r_fov.upDegrees;
        data.right_eye_parameters.field_of_view.right_degrees = r_fov.rightDegrees;
        data.right_eye_parameters.field_of_view.down_degrees = r_fov.downDegrees;
        data.right_eye_parameters.field_of_view.left_degrees = r_fov.leftDegrees;

        data
    }

    fn immediate_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData {
        let sys = &self.system_state;

        let mut data = VRFrameData::default();

        data.pose.position = Some(sys.sensorState.pose.position);
        data.pose.orientation = Some(sys.sensorState.pose.orientation);
        data.left_view_matrix = sys.sensorState.leftViewMatrix;
        data.right_view_matrix = sys.sensorState.rightViewMatrix;

        let right_handed = sys.controllerState[0].hand == mozgfx::ControllerHand_Right;

        let proj = |fov: mozgfx::VRFieldOfView| -> [f32; 16] {
            use std::f64::consts::PI;

            let up_tan = (fov.upDegrees * PI / 180.0).tan();
            let down_tan = (fov.downDegrees * PI / 180.0).tan();
            let left_tan = (fov.leftDegrees * PI / 180.0).tan();
            let right_tan = (fov.rightDegrees * PI / 180.0).tan();
            let handedness_scale = if right_handed { -1.0 } else { 1.0 };
            let pxscale = 2.0 / (left_tan + right_tan);
            let pxoffset = (left_tan - right_tan) * pxscale * 0.5;
            let pyscale = 2.0 / (up_tan + down_tan);
            let pyoffset = (up_tan - down_tan) * pyscale * 0.5;
            let mut m = [0.0f32; 16];
            m[0 * 4 + 0] = pxscale as f32;
            m[1 * 4 + 1] = pyscale as f32;
            m[2 * 4 + 0] = (pxoffset * handedness_scale) as f32;
            m[2 * 4 + 1] = (-pyoffset * handedness_scale) as f32;
            m[2 * 4 + 2] = (far_z / (near_z - far_z) * -handedness_scale) as f32;
            m[2 * 4 + 3] = handedness_scale as f32;
            m[3 * 4 + 2] = ((far_z * near_z) / (near_z - far_z)) as f32;
            m
        };

        let left_fov =
            sys.displayState.mEyeFOV[mozgfx::VRDisplayState_Eye_Eye_Left as usize];
        let right_fov =
            sys.displayState.mEyeFOV[mozgfx::VRDisplayState_Eye_Eye_Right as usize];

        data.left_projection_matrix = proj(left_fov);
        data.right_projection_matrix = proj(right_fov);

        data.timestamp = sys.sensorState.timestamp;

        data
    }

    fn synced_frame_data(&self, near_z: f64, far_z: f64) -> VRFrameData {
        self.immediate_frame_data(near_z, far_z)
    }

    fn reset_pose(&mut self) {
    }

    fn sync_poses(&mut self) {
        if !self.presenting {
            self.start_present(None);
        }

        let last_frame_id = self.system_state.displayState.mLastSubmittedFrameId;
        let last_pres_gen = self.system_state.displayState.mPresentingGeneration;
        let sys = self.shmem.as_mut().pull_system(&|sys| {
            sys.displayState.mLastSubmittedFrameId >= last_frame_id ||
                sys.displayState.mSuppressFrames ||
                !sys.displayState.mIsConnected
        });
        if sys.displayState.mPresentingGeneration != last_pres_gen {
            self.events.push(VRDisplayEvent::Exit(0));
        } else {
            self.system_state = sys;
        }
    }

    fn bind_framebuffer(&mut self, _index: u32) {
    }

    fn get_framebuffers(&self) -> Vec<VRFramebuffer> {
        let rendered_layer = self.rendered_layer.as_ref().unwrap();
        let l = rendered_layer.left_bounds;
        let r = rendered_layer.right_bounds;
        vec![
            VRFramebuffer {
                eye_index: 0,
                attributes: self.attributes,
                viewport: VRViewport::new(l[0] as i32, l[1] as i32, l[2] as i32, l[3] as i32),
            },
            VRFramebuffer {
                eye_index: 1,
                attributes: self.attributes,
                viewport: VRViewport::new(r[0] as i32, r[1] as i32, r[2] as i32, r[3] as i32),
            },
        ]
    }

    fn render_layer(&mut self, layer: &VRLayer) {
        self.rendered_layer = Some(layer.clone());
    }

    fn submit_frame(&mut self) {
        let layer_stereo_immersive = {
            let rendered_layer = self.rendered_layer.as_ref().unwrap();
            mozgfx::VRLayer_Stereo_Immersive {
                mTextureHandle: rendered_layer.texture_id as u64,
                mTextureType: mozgfx::VRLayerTextureType_LayerTextureType_GeckoSurfaceTexture,
                mFrameId: self.system_state.sensorState.inputFrameID,
                mLeftEyeRect: mozgfx::VRLayerEyeRect {
                    x: rendered_layer.left_bounds[0],
                    y: rendered_layer.left_bounds[1],
                    width: rendered_layer.left_bounds[2],
                    height: rendered_layer.left_bounds[3],
                },
                mRightEyeRect: mozgfx::VRLayerEyeRect {
                    x: rendered_layer.right_bounds[0],
                    y: rendered_layer.right_bounds[1],
                    width: rendered_layer.right_bounds[2],
                    height: rendered_layer.right_bounds[3],
                },
                mInputFrameId: 0,
                ..mozgfx::VRLayer_Stereo_Immersive::default()
            }
        };

        let layer = mozgfx::VRLayerState {
            type_: mozgfx::VRLayerType_LayerType_Stereo_Immersive,
            __bindgen_anon_1: mozgfx::VRLayerState__bindgen_ty_1 {
                layer_stereo_immersive,
            },
            ..mozgfx::VRLayerState::default()
        };

        self.browser_state.layerState[0] = layer;
        self.push_browser();
    }

    fn start_present(&mut self, attributes: Option<VRFramebufferAttributes>) {
        if self.presenting {
            return;
        }
        self.presenting = true;
        if let Some(attributes) = attributes {
            self.attributes = attributes;
        }
        self.browser_state.layerState[0].type_ = mozgfx::VRLayerType_LayerType_Stereo_Immersive;
        let count = self.browser_state.layerState.len();
        for i in 1..count {
            self.browser_state.layerState[i].type_ = mozgfx::VRLayerType_LayerType_None;
        }
        self.browser_state.presentationActive = true;
        self.push_browser();
    }

    fn stop_present(&mut self) {
        if !self.presenting {
            return;
        }
        self.browser_state.presentationActive = false;
        self.push_browser();
    }
}
