use {
    VRDisplay, VRDisplayCapabilities, VRDisplayData, VRFrameData, VRFramebuffer, VRGamepadPtr,
    VRLayer
};
use openxr::{Session, FormFactor, FrameStream, Graphics, Instance, ViewConfigurationType};
use openxr::d3d::{D3D11, Requirements, SessionCreateInfo};
use openxr::sys::platform::{ID3D11Device};
use std::{mem, ptr};
use winapi::Interface;
use winapi::shared::dxgi;
use winapi::shared::winerror::{DXGI_ERROR_NOT_FOUND, S_OK};
use winapi::um::d3d11;
use winapi::um::d3dcommon::*;
use wio::com::ComPtr;

pub struct OpenXrDisplay {
    session: Session<D3D11>,
    frame_stream: FrameStream<D3D11>,
    device: ComPtr<ID3D11Device>,
    device_context: ComPtr<d3d11::ID3D11DeviceContext>,
}

unsafe impl Send for OpenXrDisplay {}
unsafe impl Sync for OpenXrDisplay {}

fn get_matching_adapter(
    requirements: &Requirements,
) -> Result<ComPtr<dxgi::IDXGIAdapter1>, String>
{
    unsafe {
        let mut factory_ptr: *mut dxgi::IDXGIFactory1 = ptr::null_mut();
        let result = dxgi::CreateDXGIFactory1(&dxgi::IDXGIFactory1::uuidof(), &mut factory_ptr as *mut _ as *mut _);
        assert_eq!(result, S_OK);
        let factory = ComPtr::from_raw(factory_ptr);

        let index = 0;
        loop {
            let mut adapter_ptr = ptr::null_mut();
            let result = factory.EnumAdapters1(index, &mut adapter_ptr);
            if result == DXGI_ERROR_NOT_FOUND {
                return Err("No matching adapter".to_owned());
            }
            assert_eq!(result, S_OK);
            let adapter = ComPtr::from_raw(adapter_ptr);
            let mut adapter_desc = mem::zeroed();
            let result = adapter.GetDesc1(&mut adapter_desc);
            assert_eq!(result, S_OK);
            let adapter_luid = &adapter_desc.AdapterLuid;
            if adapter_luid.LowPart == requirements.adapter_luid.LowPart &&
                adapter_luid.HighPart == requirements.adapter_luid.HighPart
            {
                return Ok(adapter);
            }
        }
    }
}

fn select_feature_levels(requirements: &Requirements) -> Vec<D3D_FEATURE_LEVEL> {
    let levels = [
        D3D_FEATURE_LEVEL_12_1,
        D3D_FEATURE_LEVEL_12_0,
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    ];
    levels
        .into_iter()
        .filter(|&&level| level >= requirements.min_feature_level)
        .map(|&level| level)
        .collect()
}

fn init_device_for_adapter(
    adapter: ComPtr<dxgi::IDXGIAdapter1>,
    feature_levels: &[D3D_FEATURE_LEVEL],
) -> Result<(ComPtr<ID3D11Device>, ComPtr<d3d11::ID3D11DeviceContext>), String>
{
    let adapter = adapter.up::<dxgi::IDXGIAdapter>();
    unsafe {
        let mut device_ptr = ptr::null_mut();
        let mut device_context_ptr = ptr::null_mut();
        let hr = d3d11::D3D11CreateDevice(
            adapter.as_raw(),
            D3D_DRIVER_TYPE_UNKNOWN,
            ptr::null_mut(),
            d3d11::D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            feature_levels.as_ptr(),
            feature_levels.len() as u32,
            d3d11::D3D11_SDK_VERSION,
            &mut device_ptr,
            ptr::null_mut(),
            &mut device_context_ptr,
        );
        assert_eq!(hr, S_OK);
        let device = ComPtr::from_raw(device_ptr);
        let device_context = ComPtr::from_raw(device_context_ptr);
        Ok((device, device_context))
    }
}

impl OpenXrDisplay {
    pub fn new(instance: &Instance) -> Result<OpenXrDisplay, String> {
        let system = instance
            .system(FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(|e| format!("{:?}", e))?;

        let requirements = D3D11::requirements(instance, system)
            .map_err(|e| format!("{:?}", e))?;
        let adapter = get_matching_adapter(&requirements)?;
        let feature_levels = select_feature_levels(&requirements);
        let (device, device_context) = init_device_for_adapter(adapter, &feature_levels)?;

        let (session, frame_stream) = unsafe {
            instance
                .create_session::<D3D11>(
                    system,
                    &SessionCreateInfo {
                        device: device.as_raw(),
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
            device,
            device_context,
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
            display_id: crate::utils::new_id(),
            display_name: "".to_owned(),
            connected: true,
            capabilities,
            stage_parameters: None,
            left_eye_parameters: panic!(), //FIXME
            right_eye_parameters: panic!(), //FIXME
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
