#![cfg(target_os="android")]
#![cfg(feature = "oculusvr")]

use {VRGamepad, VRGamepadButton, VRGamepadData, VRGamepadHand, VRGamepadState};
use ovr_mobile_sys as ovr;
use ovr_mobile_sys::ovrButton::*;
use ovr_mobile_sys::ovrControllerCapabilties::*;
use ovr_mobile_sys::ovrControllerType::*;
use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::sync::Arc;
use super::display::{ovr_quat_to_array, ovr_vec3_to_array};
use super::super::utils;

pub type OculusVRGamepadPtr = Arc<RefCell<OculusVRGamepad>>;

pub struct OculusVRGamepad {
    ovr: *mut ovr::ovrMobile,
    ovr_id: ovr::ovrDeviceID,
    ovr_type: ovr::ovrControllerType,
    connected: bool,
    capabilities: InputCapabilities,
    gamepad_id: u32,
    display_id: u32,
}

unsafe impl Send for OculusVRGamepad {}
unsafe impl Sync for OculusVRGamepad {}

impl OculusVRGamepad {
    pub fn new(ovr: *mut ovr::ovrMobile,
               ovr_id: ovr::ovrDeviceID,
               ovr_type: ovr::ovrControllerType,
               display_id: u32)
               -> Arc<RefCell<OculusVRGamepad>>
    {
        let capabilities = InputCapabilities::from_ovr(ovr, ovr_type, ovr_id);

        let gamepad = Self {
            ovr: ovr,
            ovr_id: ovr_id,
            ovr_type: ovr_type,
            connected: true,
            capabilities: capabilities.unwrap_or_default(),
            gamepad_id: utils::new_id(),
            display_id: display_id,
        };

        Arc::new(RefCell::new(gamepad))
    }

    pub fn refresh_available_gamepads(ovr: *mut ovr::ovrMobile,
                                      display_id: u32,
                                      out: &mut Vec<OculusVRGamepadPtr>) {
        let mut index = 0;
        // Reset connected status
        for gamepad in out.iter() {
            gamepad.borrow_mut().connected = false;
        }

        loop {
            let mut caps: ovr::ovrInputCapabilityHeader = unsafe { mem::uninitialized() };

            if unsafe { ovr::vrapi_EnumerateInputDevices(ovr, index, &mut caps) } < 0 {
                // No more input devices to enumerate
                break;
            }

            index += 1;

            if caps.Type != ovrControllerType_TrackedRemote && caps.Type != ovrControllerType_Headset  {
                // Not interested in this kind of input device
                continue;
            }

            // Update if the controller type already exists
            if let Some(gamepad) = out.iter().find(|g| g.borrow().ovr_type == caps.Type).as_ref() {
                let mut gamepad = gamepad.borrow_mut();
                gamepad.ovr = ovr;
                gamepad.ovr_id = caps.DeviceID;
                gamepad.connected = true;
                gamepad.update_capabilities();
                continue;
            }

            // Create new Gamepad instance
            let gamepad = OculusVRGamepad::new(ovr, caps.DeviceID, caps.Type, display_id);
            out.push(gamepad);
        }
    }

    pub fn update_capabilities(&mut self) {
        if let Ok(capabilities) = InputCapabilities::from_ovr(self.ovr, self.ovr_type, self.ovr_id) {
            self.capabilities = capabilities;
        }
    }

    // Sensor input is only available while in VR mode.
    pub fn on_exit_vrmode(&mut self) {
        self.connected = false;
        self.ovr = ptr::null_mut();
    }

    fn fetch_axes(&self, touching: bool, pos: &ovr::ovrVector2f, out: &mut VRGamepadState) {
        // Axes
        // Touchpad: (0,0) is the top-left corner.
        // Map to -1 1 for each axis.
        let x = pos.x / self.capabilities.trackpad_max_x as f32;
        let y = pos.y / self.capabilities.trackpad_max_y as f32;
        out.axes = if touching {
            [x as f64 * 2.0 - 1.0, 
             y as f64 * 2.0 - 1.0].to_vec()
        } else {
            [0.0, 0.0].to_vec()
        };
    }

    fn fetch_remote_controller_state(&self, out: &mut VRGamepadState) {
        let mut state: ovr::ovrInputStateTrackedRemote = unsafe { mem::zeroed() };
        state.Header.ControllerType = ovrControllerType_TrackedRemote;
        unsafe {
            ovr::vrapi_GetCurrentInputState(self.ovr, self.ovr_id, &mut state.Header);
        }
        let touching_trackpad = state.TrackpadStatus > 0;

        // Axes
        self.fetch_axes(touching_trackpad, &state.TrackpadPosition, out);

        // 0 - Trackpad
        out.buttons.push(VRGamepadButton::new(touching_trackpad));

        // 1 - Trigger A
        out.buttons.push(VRGamepadButton::new(state.Buttons & (ovrButton_A as u32) > 0));
    }

    fn fetch_headset_controller_state(&self, out: &mut VRGamepadState) {
        let mut state: ovr::ovrInputStateHeadset = unsafe { mem::zeroed() };
        state.Header.ControllerType = ovrControllerType_Headset;
        unsafe {
            ovr::vrapi_GetCurrentInputState(self.ovr, self.ovr_id, &mut state.Header);
        }
        let touching_trackpad = state.TrackpadStatus > 0;

        // Axes
        self.fetch_axes(touching_trackpad, &state.TrackpadPosition, out);

        // 0 - Trackpad
        out.buttons.push(VRGamepadButton::new(touching_trackpad));

        // 1 - Trigger A
        out.buttons.push(VRGamepadButton::new(state.Buttons & (ovrButton_A as u32) > 0));
    }

    fn fetch_tracking_state(&self, out: &mut VRGamepadState) {
        let mut tracking: ovr::ovrTracking = unsafe { mem::uninitialized() };
        let status = unsafe {
             ovr::vrapi_GetInputTrackingState(self.ovr, self.ovr_id, 0.0, &mut tracking)
        };

        if status != ovr::ovrSuccessResult::ovrSuccess as i32 {
            out.connected = false;
            return;
        }

        if self.capabilities.controller_capabilities & (ovrControllerCaps_HasOrientationTracking as u32) > 0 {
            out.pose.orientation = Some(ovr_quat_to_array(&tracking.HeadPose.Pose.Orientation));
        }

        if self.capabilities.controller_capabilities & (ovrControllerCaps_HasPositionTracking as u32) > 0 {
            out.pose.position = Some(ovr_vec3_to_array(&tracking.HeadPose.Pose.Position));
        }
    }
}

impl VRGamepad for OculusVRGamepad {
    fn id(&self) -> u32 {
        self.gamepad_id
    }

    fn data(&self) -> VRGamepadData {
        let name = if self.ovr_type == ovrControllerType_TrackedRemote {
            "Gear VR Remote Controller"
        } else {
            "Gear VR Headset Controller"
        };

        let hand = if self.capabilities.controller_capabilities & (ovrControllerCaps_RightHand as u32) > 0 {
            VRGamepadHand::Right
        } else if self.capabilities.controller_capabilities & (ovrControllerCaps_LeftHand as u32) > 0 {
            VRGamepadHand::Left
        } else {
            VRGamepadHand::Unknown
        };

        VRGamepadData {
            display_id: self.display_id,
            name: name.into(),
            hand: hand,
        }
    }

    fn state(&self) -> VRGamepadState {
        let mut out = VRGamepadState::default();

        out.gamepad_id = self.gamepad_id;
        out.connected = self.connected && !self.ovr.is_null();

        if out.connected {
            if self.ovr_type == ovrControllerType_TrackedRemote {
                self.fetch_remote_controller_state(&mut out);
            } else {
                self.fetch_headset_controller_state(&mut out);
            }
            self.fetch_tracking_state(&mut out);
        }

        out
    }
}

struct InputCapabilities {
    controller_capabilities: u32,
    trackpad_max_x: u16,
    trackpad_max_y: u16,
}

impl InputCapabilities {
    pub fn from_ovr(ovr: *mut ovr::ovrMobile,
                    ovr_type: ovr::ovrControllerType,
                    ovr_id: ovr::ovrDeviceID) -> Result<InputCapabilities,()> {
        if ovr_type == ovrControllerType_TrackedRemote {
            let mut caps: ovr::ovrInputTrackedRemoteCapabilities = unsafe { mem::uninitialized() };
            caps.Header.DeviceID = ovr_id;
            caps.Header.Type = ovr_type;

            let status = unsafe {
                ovr::vrapi_GetInputDeviceCapabilities(ovr, &mut caps.Header)
            };

            if status != ovr::ovrSuccessResult::ovrSuccess as i32 {
                return Err(());
            }

            Ok(Self {
                controller_capabilities: caps.ControllerCapabilities,
                trackpad_max_x: caps.TrackpadMaxX,
                trackpad_max_y: caps.TrackpadMaxY,
            })

        } else {
            let mut caps: ovr::ovrInputHeadsetCapabilities = unsafe { mem::uninitialized() };
            caps.Header.DeviceID = ovr_id;
            caps.Header.Type = ovr_type;

            let status = unsafe {
                ovr::vrapi_GetInputDeviceCapabilities(ovr, &mut caps.Header)
            };
            
            if status != ovr::ovrSuccessResult::ovrSuccess as i32 {
                return Err(());
            }

            Ok(Self {
                controller_capabilities: caps.ControllerCapabilities,
                trackpad_max_x: caps.TrackpadMaxX,
                trackpad_max_y: caps.TrackpadMaxY,
            })
        }
    }
}

impl Default for InputCapabilities {
    fn default() -> InputCapabilities {
        InputCapabilities {
            controller_capabilities: 0,
            trackpad_max_x: 299,
            trackpad_max_y: 199,
        }
    }
}
