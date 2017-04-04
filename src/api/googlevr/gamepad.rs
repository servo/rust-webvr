#![cfg(feature = "googlevr")]
use {VRGamepad, VRGamepadData, VRGamepadState, VRGamepadButton};
use super::super::utils;
use gvr_sys as gvr;
use gvr_sys::gvr_controller_api_status::*;
use gvr_sys::gvr_controller_button::*;
use gvr_sys::gvr_controller_connection_state::*;
use std::cell::RefCell;
use std::mem;
use std::ffi::CStr;
use std::sync::Arc;

pub type GoogleVRGamepadPtr = Arc<RefCell<GoogleVRGamepad>>;

pub struct GoogleVRGamepad {
    ctx: *mut gvr::gvr_controller_context,
    state: *mut gvr::gvr_controller_state,
    gamepad_id: u64,
    display_id: u64
}

unsafe impl Send for GoogleVRGamepad {}
unsafe impl Sync for GoogleVRGamepad {}

impl GoogleVRGamepad {
    pub unsafe fn new(ctx: *mut gvr::gvr_controller_context,
                      display_id: u64)
                      -> Result<Arc<RefCell<GoogleVRGamepad>>, String> {
        let gamepad = Self {
            ctx: ctx,
            state: gvr::gvr_controller_state_create(),
            gamepad_id: utils::new_id(),
            display_id: display_id
        };
        gvr::gvr_controller_state_update(ctx, 0, gamepad.state);
        let api_status = gvr::gvr_controller_state_get_api_status(gamepad.state);
        if api_status != GVR_CONTROLLER_API_OK as i32 {
            let message = CStr::from_ptr(gvr::gvr_controller_api_status_to_string(api_status));
            return Err(message.to_string_lossy().into());
        }

        Ok(Arc::new(RefCell::new(gamepad)))
    }
}

impl Drop for GoogleVRGamepad {
    fn drop(&mut self) {
        unsafe {
            gvr::gvr_controller_state_destroy(mem::transmute(&self.state));
        }
    }
}

impl VRGamepad for GoogleVRGamepad {
    fn id(&self) -> u64 {
        self.gamepad_id
    }

    fn data(&self) -> VRGamepadData {
        VRGamepadData {
            display_id: self.display_id,
            name: "GoogleVR DayDream".into()
        }
    }

    fn state(&self) -> VRGamepadState {
        let mut out = VRGamepadState::default();

        out.gamepad_id = self.gamepad_id;
        unsafe {
            gvr::gvr_controller_state_update(self.ctx, 0, self.state);
            let connection_state = gvr::gvr_controller_state_get_connection_state(self.state);
            out.connected = connection_state == GVR_CONTROLLER_CONNECTED as i32;

            let touchpad_touching = gvr::gvr_controller_state_is_touching(self.state);

            // Touchpad: (0,0) is the top-left of the touchpad and (1,1)
            // Map to -1 1 for each axis.
            let pos = gvr::gvr_controller_state_get_touch_pos(self.state);
            out.axes = if touchpad_touching {
                [pos.x as f64 * 2.0 - 1.0, 
                 pos.y as f64 * 2.0 - 1.0].to_vec()
            } else {
                [0.0, 0.0].to_vec()
            };

            // Add touchpad as a button
            out.buttons.push(VRGamepadButton {
                pressed: gvr::gvr_controller_state_get_button_state(self.state, GVR_CONTROLLER_BUTTON_CLICK as i32),
                touched: touchpad_touching,
            });

            // Extra buttons
            let buttons = [GVR_CONTROLLER_BUTTON_HOME,
                           GVR_CONTROLLER_BUTTON_APP,
                           GVR_CONTROLLER_BUTTON_VOLUME_UP,
                           GVR_CONTROLLER_BUTTON_VOLUME_DOWN];
            for button in &buttons {
                let pressed = gvr::gvr_controller_state_get_button_state(self.state, *button as i32);
                out.buttons.push(VRGamepadButton {
                    pressed: pressed,
                    touched: pressed,
                }); 
            }

            let quat = gvr::gvr_controller_state_get_orientation(self.state);
            out.pose.orientation = Some([
                quat.qx, quat.qy, quat.qz, quat.qw
            ]);

            let acc = gvr::gvr_controller_state_get_accel(self.state);
            out.pose.linear_acceleration = Some([
                acc.x, acc.y, acc.z
            ]);

            let vel = gvr::gvr_controller_state_get_gyro(self.state);
            out.pose.angular_velocity = Some([
                vel.x, vel.y, vel.z
            ]);
        }

        out
    }
}
