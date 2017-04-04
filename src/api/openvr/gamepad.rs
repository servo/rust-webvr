use super::binding as openvr;
use super::binding::ETrackingUniverseOrigin::*;
use super::binding::EVRButtonId;
use super::binding::EVRButtonId::*;
use super::display::OpenVRDisplay;
use super::super::utils;
use std::cell::RefCell;
use std::mem;
use std::sync::Arc;
use {VRGamepad, VRGamepadData, VRGamepadHand, VRGamepadState, VRGamepadButton};

pub type OpenVRGamepadPtr = Arc<RefCell<OpenVRGamepad>>;

pub struct OpenVRGamepad {
    gamepad_id: u64,
    display_id: u64,
    index: openvr::TrackedDeviceIndex_t,
    system: *mut openvr::VR_IVRSystem_FnTable
}

unsafe impl Send for OpenVRGamepad {}
unsafe impl Sync for OpenVRGamepad {}

impl OpenVRGamepad {
    pub fn new(index: openvr::TrackedDeviceIndex_t,
               system: *mut openvr::VR_IVRSystem_FnTable,
               display_id: u64)
               -> Arc<RefCell<OpenVRGamepad>> {
        Arc::new(RefCell::new(OpenVRGamepad {
            gamepad_id: utils::new_id(),
            display_id: display_id,
            index: index,
            system: system
        }))
    }

    pub fn index(&self) -> openvr::TrackedDeviceIndex_t {
        self.index
    }
}

impl VRGamepad for OpenVRGamepad {
    fn id(&self) -> u64 {
        self.gamepad_id
    }

    fn data(&self) -> VRGamepadData {
        VRGamepadData {
            display_id: self.display_id,
            name: format!("OpenVR {:?}", self.index),
            hand: VRGamepadHand::Unknown
        }
    }
    
    fn state(&self) -> VRGamepadState {
        let mut state = VRGamepadState::default();

        state.gamepad_id = self.gamepad_id;
        let mut controller: openvr::VRControllerState_t = unsafe { mem::uninitialized() };
        let mut tracked_poses: [openvr::TrackedDevicePose_t; openvr::k_unMaxTrackedDeviceCount as usize]
                              = unsafe { mem::uninitialized() };

        unsafe {
            (*self.system).GetControllerState.unwrap()(self.index,
                                                       &mut controller,
                                                       mem::size_of::<openvr::VRControllerState_t>() as u32);
            (*self.system).GetDeviceToAbsoluteTrackingPose.unwrap()(ETrackingUniverseOrigin_TrackingUniverseSeated,
                                                                    0.04f32,
                                                                    &mut tracked_poses[0],
                                                                    openvr::k_unMaxTrackedDeviceCount);
        }
        let pose = &tracked_poses[self.index as usize];

        state.connected = pose.bDeviceIsConnected;

        let trackpad = controller.rAxis[0];
        // Analog trigger data is in only the X axis
        let trigger = controller.rAxis[1];
        state.axes = [trackpad.x as f64, trackpad.y as f64, trigger.x as f64].to_vec();

        // TODO: check spec order
        let buttons = [
            button_mask(EVRButtonId_k_EButton_Grip),
            button_mask(EVRButtonId_k_EButton_ApplicationMenu) 
        ];

        for mask in buttons.iter() {
            state.buttons.push(VRGamepadButton {
                pressed: (controller.ulButtonPressed & mask) != 0,
                touched: (controller.ulButtonTouched & mask) != 0
            });
        }

        OpenVRDisplay::fetch_pose(&pose, &mut state.pose);

        state
    }
}

#[inline]
fn button_mask(id: EVRButtonId) -> u64 {
    1u64 << (id as u32)
}
