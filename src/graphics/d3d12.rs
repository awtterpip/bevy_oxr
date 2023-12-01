use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

// use anyhow::Context;
use bevy::math::uvec2;
use bevy::prelude::*;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use eyre::{Context, ContextCompat};
use openxr as xr;
use wgpu::Instance;
use wgpu_hal::{api::Dx12, Api};
use wgpu_hal::{Adapter as HalAdapter, Instance as HalInstance};
use winapi::shared::dxgiformat::{self, DXGI_FORMAT};
use winapi::um::{d3d12 as winapi_d3d12, d3dcommon};
use xr::EnvironmentBlendMode;

use crate::graphics::extensions::XrExtensions;
use crate::input::XrInput;

use crate::resources::{
    OXrSessionSetupInfo, Swapchain, SwapchainInner, XrEnvironmentBlendMode, XrFormat, XrFrameState,
    XrFrameWaiter, XrInstance, XrResolution, XrSession, XrSessionRunning, XrSwapchain, XrViews,
};

#[cfg(all(feature = "d3d12", windows))]
use crate::resources::D3D12OXrSessionSetupInfo;
#[cfg(feature = "vulkan")]
use crate::resources::VulkanOXrSessionSetupInfo;

use super::{XrAppInfo, XrPreferdBlendMode};
use crate::VIEW_TYPE;

pub fn initialize_xr_instance(
    window: Option<RawHandleWrapper>,
    xr_entry: xr::Entry,
    reqeusted_extensions: XrExtensions,
    available_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
) -> eyre::Result<(
    XrInstance,
    OXrSessionSetupInfo,
    XrEnvironmentBlendMode,
    RenderDevice,
    RenderQueue,
    RenderAdapterInfo,
    RenderAdapter,
    Instance,
)> {
    #[cfg(target_os = "android")]
    xr_entry.initialize_android_loader()?;

    assert!(available_extensions.raw().khr_d3d12_enable);
    //info!("available xr exts: {:#?}", available_extensions);

    let mut enabled_extensions: xr::ExtensionSet =
        (available_extensions & reqeusted_extensions).into();
    enabled_extensions.khr_d3d12_enable = true;

    let available_layers = xr_entry.enumerate_layers()?;
    //info!("available xr layers: {:#?}", available_layers);

    let xr_instance = xr_entry.create_instance(
        &xr::ApplicationInfo {
            application_name: &app_info.name,
            engine_name: "Bevy",
            ..Default::default()
        },
        &enabled_extensions,
        &[],
    )?;
    info!("created instance");
    let instance_props = xr_instance.properties()?;
    let xr_system_id = xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    info!("created system");
    let system_props = xr_instance.system_properties(xr_system_id).unwrap();
    info!(
        "loaded OpenXR runtime: {} {} {}",
        instance_props.runtime_name,
        instance_props.runtime_version,
        if system_props.system_name.is_empty() {
            "<unnamed>"
        } else {
            &system_props.system_name
        }
    );

    let blend_modes = xr_instance.enumerate_environment_blend_modes(xr_system_id, VIEW_TYPE)?;
    let blend_mode: EnvironmentBlendMode = match prefered_blend_mode {
        XrPreferdBlendMode::Opaque if blend_modes.contains(&EnvironmentBlendMode::OPAQUE) => {
            bevy::log::info!("Using Opaque");
            EnvironmentBlendMode::OPAQUE
        }
        XrPreferdBlendMode::Additive if blend_modes.contains(&EnvironmentBlendMode::ADDITIVE) => {
            bevy::log::info!("Using Additive");
            EnvironmentBlendMode::ADDITIVE
        }
        XrPreferdBlendMode::AlphaBlend
            if blend_modes.contains(&EnvironmentBlendMode::ALPHA_BLEND) =>
        {
            bevy::log::info!("Using AlphaBlend");
            EnvironmentBlendMode::ALPHA_BLEND
        }
        _ => {
            bevy::log::info!("Using Opaque");
            EnvironmentBlendMode::OPAQUE
        }
    };

    // wgpu hardcodes this, so we'll hardcode it here too
    let d3d_target_version: u32 = d3dcommon::D3D_FEATURE_LEVEL_11_0;

    let reqs = xr_instance.graphics_requirements::<xr::D3D12>(xr_system_id)?;
    if (d3d_target_version) < (reqs.min_feature_level as u32) {
        panic!(
            "OpenXR runtime requires D3D12 feature level >= {}",
            reqs.min_feature_level
        );
    }
    let instance_descriptor = &wgpu_hal::InstanceDescriptor {
        name: &app_info.name,
        dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
        flags: wgpu::InstanceFlags::from_build_config().with_env(),
        gles_minor_version: Default::default(),
    };
    let wgpu_raw_instance: wgpu_hal::dx12::Instance =
        unsafe { wgpu_hal::dx12::Instance::init(instance_descriptor)? };
    let wgpu_adapters: Vec<wgpu_hal::ExposedAdapter<wgpu_hal::dx12::Api>> =
        unsafe { wgpu_raw_instance.enumerate_adapters() };
    let wgpu_exposed_adapter = wgpu_adapters
        .into_iter()
        .find(|a| {
            let mut desc = unsafe { std::mem::zeroed() };
            unsafe { a.adapter.raw_adapter().GetDesc1(&mut desc) };
            desc.AdapterLuid.HighPart == reqs.adapter_luid.HighPart
                && desc.AdapterLuid.LowPart == reqs.adapter_luid.LowPart
        })
        .context("Failed to find DXGI adapter matching LUID provided by runtime")?;

    let wgpu_instance =
        unsafe { wgpu::Instance::from_hal::<wgpu_hal::api::Dx12>(wgpu_raw_instance) };

    let wgpu_features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::MULTIVIEW
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::MULTI_DRAW_INDIRECT;

    let wgpu_limits = wgpu::Limits {
        max_bind_groups: 8,
        max_storage_buffer_binding_size: wgpu_exposed_adapter
            .capabilities
            .limits
            .max_storage_buffer_binding_size,
        max_push_constant_size: 4,
        ..Default::default()
    };

    let wgpu_open_device = unsafe {
        wgpu_exposed_adapter
            .adapter
            .open(wgpu_features, &wgpu_limits)?
    };

    let wgpu_adapter = unsafe { wgpu_instance.create_adapter_from_hal(wgpu_exposed_adapter) };
    let raw_device = wgpu_open_device.device.raw_device().as_mut_ptr();
    let raw_queue = wgpu_open_device.device.raw_queue().as_mut_ptr();
    let (wgpu_device, wgpu_queue) = unsafe {
        wgpu_adapter.create_device_from_hal(
            wgpu_open_device,
            &wgpu::DeviceDescriptor {
                label: Some("bevy_oxr device"),
                required_features: wgpu_features,
                required_limits: wgpu_limits,
            },
            None,
        )?
    };

    Ok((
        xr_instance.into(),
        OXrSessionSetupInfo::D3D12(D3D12OXrSessionSetupInfo {
            raw_device,
            raw_queue,
            xr_system_id,
        }),
        blend_mode.into(),
        wgpu_device.into(),
        RenderQueue(wgpu_queue.into()),
        RenderAdapterInfo(wgpu_adapter.get_info()),
        RenderAdapter(wgpu_adapter.into()),
        wgpu_instance.into(),
    ))
}

pub fn start_xr_session(
    window: Option<RawHandleWrapper>,
    ptrs: &OXrSessionSetupInfo,
    xr_instance: &XrInstance,
    render_device: &RenderDevice,
    render_adapter: &RenderAdapter,
    wgpu_instance: &Instance,
) -> eyre::Result<(
    XrSession,
    XrResolution,
    XrFormat,
    XrSessionRunning,
    XrFrameWaiter,
    XrSwapchain,
    XrInput,
    XrViews,
    XrFrameState,
)> {
    let wgpu_device = render_device.wgpu_device();
    let wgpu_adapter = &render_adapter.0;

    #[allow(unreachable_patterns)]
    let setup_info = match ptrs {
        OXrSessionSetupInfo::D3D12(v) => v,
        _ => eyre::bail!("Wrong Graphics Api"),
    };
    let (session, frame_wait, frame_stream) = unsafe {
        xr_instance.create_session::<xr::D3D12>(
            setup_info.xr_system_id,
            &xr::d3d::SessionCreateInfoD3D12 {
                device: setup_info.raw_device.cast(),
                queue: setup_info.raw_queue.cast(),
            },
        )
    }?;

    let views =
        xr_instance.enumerate_view_configuration_views(setup_info.xr_system_id, VIEW_TYPE)?;
    let surface = window.map(|wrapper| unsafe {
        // SAFETY: Plugins should be set up on the main thread.
        let handle = wrapper.get_handle();
        wgpu_instance
            .create_surface(handle)
            .expect("Failed to create wgpu surface")
    });
    let swapchain_format = surface
        .as_ref()
        .map(|surface| surface.get_capabilities(wgpu_adapter).formats[0])
        .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

    // TODO: Log swapchain format

    let resolution = uvec2(
        views[0].recommended_image_rect_width,
        views[0].recommended_image_rect_height,
    );

    let handle = session
        .create_swapchain(&xr::SwapchainCreateInfo {
            create_flags: xr::SwapchainCreateFlags::EMPTY,
            usage_flags: xr::SwapchainUsageFlags::COLOR_ATTACHMENT
                | xr::SwapchainUsageFlags::SAMPLED,
            format: wgpu_to_d3d12(swapchain_format).expect("unsupported texture format"),
            // The Vulkan graphics pipeline we create is not set up for multisampling,
            // so we hardcode this to 1. If we used a proper multisampling setup, we
            // could set this to `views[0].recommended_swapchain_sample_count`.
            sample_count: 1,
            width: resolution.x,
            height: resolution.y,
            face_count: 1,
            array_size: 2,
            mip_count: 1,
        })
        .unwrap();

    let images = handle.enumerate_images().unwrap();

    let buffers = images
        .into_iter()
        .map(|color_image| {
            info!("image map swapchain");
            let wgpu_hal_texture = unsafe {
                <Dx12 as Api>::Device::texture_from_raw(
                    d3d12::ComPtr::from_raw(color_image as *mut _),
                    swapchain_format,
                    wgpu::TextureDimension::D2,
                    wgpu::Extent3d {
                        width: resolution.x,
                        height: resolution.y,
                        depth_or_array_layers: 2,
                    },
                    1,
                    1,
                )
            };
            let texture = unsafe {
                wgpu_device.create_texture_from_hal::<Dx12>(
                    wgpu_hal_texture,
                    &wgpu::TextureDescriptor {
                        label: Some("VR Swapchain"),
                        size: wgpu::Extent3d {
                            width: resolution.x,
                            height: resolution.y,
                            depth_or_array_layers: 2,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: swapchain_format,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::COPY_DST,
                        view_formats: &[],
                    },
                )
            };
            texture
        })
        .collect();

    Ok((
        XrSession::D3D12(session.clone()),
        resolution.into(),
        swapchain_format.into(),
        // TODO: this shouldn't be in here
        AtomicBool::new(false).into(),
        frame_wait.into(),
        Swapchain::D3D12(SwapchainInner {
            stream: Mutex::new(frame_stream),
            handle: Mutex::new(handle),
            buffers,
            image_index: Mutex::new(0),
        })
        .into(),
        XrInput::new(xr_instance, &session.into_any_graphics())?,
        Vec::default().into(),
        // TODO: Feels wrong to return a FrameState here, we probably should just wait for the next frame
        xr::FrameState {
            predicted_display_time: xr::Time::from_nanos(1),
            predicted_display_period: xr::Duration::from_nanos(1),
            should_render: true,
        }
        .into(),
    ))
}

fn wgpu_to_d3d12(format: wgpu::TextureFormat) -> Option<DXGI_FORMAT> {
    // Copied wholesale from:
    // https://github.com/gfx-rs/wgpu/blob/v0.19/wgpu-hal/src/auxil/dxgi/conv.rs#L12-L94
    // license: MIT OR Apache-2.0
    use wgpu::TextureFormat as Tf;
    use winapi::shared::dxgiformat::*;

    Some(match format {
        Tf::R8Unorm => DXGI_FORMAT_R8_UNORM,
        Tf::R8Snorm => DXGI_FORMAT_R8_SNORM,
        Tf::R8Uint => DXGI_FORMAT_R8_UINT,
        Tf::R8Sint => DXGI_FORMAT_R8_SINT,
        Tf::R16Uint => DXGI_FORMAT_R16_UINT,
        Tf::R16Sint => DXGI_FORMAT_R16_SINT,
        Tf::R16Unorm => DXGI_FORMAT_R16_UNORM,
        Tf::R16Snorm => DXGI_FORMAT_R16_SNORM,
        Tf::R16Float => DXGI_FORMAT_R16_FLOAT,
        Tf::Rg8Unorm => DXGI_FORMAT_R8G8_UNORM,
        Tf::Rg8Snorm => DXGI_FORMAT_R8G8_SNORM,
        Tf::Rg8Uint => DXGI_FORMAT_R8G8_UINT,
        Tf::Rg8Sint => DXGI_FORMAT_R8G8_SINT,
        Tf::Rg16Unorm => DXGI_FORMAT_R16G16_UNORM,
        Tf::Rg16Snorm => DXGI_FORMAT_R16G16_SNORM,
        Tf::R32Uint => DXGI_FORMAT_R32_UINT,
        Tf::R32Sint => DXGI_FORMAT_R32_SINT,
        Tf::R32Float => DXGI_FORMAT_R32_FLOAT,
        Tf::Rg16Uint => DXGI_FORMAT_R16G16_UINT,
        Tf::Rg16Sint => DXGI_FORMAT_R16G16_SINT,
        Tf::Rg16Float => DXGI_FORMAT_R16G16_FLOAT,
        Tf::Rgba8Unorm => DXGI_FORMAT_R8G8B8A8_UNORM,
        Tf::Rgba8UnormSrgb => DXGI_FORMAT_R8G8B8A8_UNORM_SRGB,
        Tf::Bgra8UnormSrgb => DXGI_FORMAT_B8G8R8A8_UNORM_SRGB,
        Tf::Rgba8Snorm => DXGI_FORMAT_R8G8B8A8_SNORM,
        Tf::Bgra8Unorm => DXGI_FORMAT_B8G8R8A8_UNORM,
        Tf::Rgba8Uint => DXGI_FORMAT_R8G8B8A8_UINT,
        Tf::Rgba8Sint => DXGI_FORMAT_R8G8B8A8_SINT,
        Tf::Rgb9e5Ufloat => DXGI_FORMAT_R9G9B9E5_SHAREDEXP,
        Tf::Rgb10a2Uint => DXGI_FORMAT_R10G10B10A2_UINT,
        Tf::Rgb10a2Unorm => DXGI_FORMAT_R10G10B10A2_UNORM,
        Tf::Rg11b10Float => DXGI_FORMAT_R11G11B10_FLOAT,
        Tf::Rg32Uint => DXGI_FORMAT_R32G32_UINT,
        Tf::Rg32Sint => DXGI_FORMAT_R32G32_SINT,
        Tf::Rg32Float => DXGI_FORMAT_R32G32_FLOAT,
        Tf::Rgba16Uint => DXGI_FORMAT_R16G16B16A16_UINT,
        Tf::Rgba16Sint => DXGI_FORMAT_R16G16B16A16_SINT,
        Tf::Rgba16Unorm => DXGI_FORMAT_R16G16B16A16_UNORM,
        Tf::Rgba16Snorm => DXGI_FORMAT_R16G16B16A16_SNORM,
        Tf::Rgba16Float => DXGI_FORMAT_R16G16B16A16_FLOAT,
        Tf::Rgba32Uint => DXGI_FORMAT_R32G32B32A32_UINT,
        Tf::Rgba32Sint => DXGI_FORMAT_R32G32B32A32_SINT,
        Tf::Rgba32Float => DXGI_FORMAT_R32G32B32A32_FLOAT,
        Tf::Stencil8 => DXGI_FORMAT_D24_UNORM_S8_UINT,
        Tf::Depth16Unorm => DXGI_FORMAT_D16_UNORM,
        Tf::Depth24Plus => DXGI_FORMAT_D24_UNORM_S8_UINT,
        Tf::Depth24PlusStencil8 => DXGI_FORMAT_D24_UNORM_S8_UINT,
        Tf::Depth32Float => DXGI_FORMAT_D32_FLOAT,
        Tf::Depth32FloatStencil8 => DXGI_FORMAT_D32_FLOAT_S8X24_UINT,
        Tf::NV12 => DXGI_FORMAT_NV12,
        Tf::Bc1RgbaUnorm => DXGI_FORMAT_BC1_UNORM,
        Tf::Bc1RgbaUnormSrgb => DXGI_FORMAT_BC1_UNORM_SRGB,
        Tf::Bc2RgbaUnorm => DXGI_FORMAT_BC2_UNORM,
        Tf::Bc2RgbaUnormSrgb => DXGI_FORMAT_BC2_UNORM_SRGB,
        Tf::Bc3RgbaUnorm => DXGI_FORMAT_BC3_UNORM,
        Tf::Bc3RgbaUnormSrgb => DXGI_FORMAT_BC3_UNORM_SRGB,
        Tf::Bc4RUnorm => DXGI_FORMAT_BC4_UNORM,
        Tf::Bc4RSnorm => DXGI_FORMAT_BC4_SNORM,
        Tf::Bc5RgUnorm => DXGI_FORMAT_BC5_UNORM,
        Tf::Bc5RgSnorm => DXGI_FORMAT_BC5_SNORM,
        Tf::Bc6hRgbUfloat => DXGI_FORMAT_BC6H_UF16,
        Tf::Bc6hRgbFloat => DXGI_FORMAT_BC6H_SF16,
        Tf::Bc7RgbaUnorm => DXGI_FORMAT_BC7_UNORM,
        Tf::Bc7RgbaUnormSrgb => DXGI_FORMAT_BC7_UNORM_SRGB,
        Tf::Etc2Rgb8Unorm
        | Tf::Etc2Rgb8UnormSrgb
        | Tf::Etc2Rgb8A1Unorm
        | Tf::Etc2Rgb8A1UnormSrgb
        | Tf::Etc2Rgba8Unorm
        | Tf::Etc2Rgba8UnormSrgb
        | Tf::EacR11Unorm
        | Tf::EacR11Snorm
        | Tf::EacRg11Unorm
        | Tf::EacRg11Snorm
        | Tf::Astc {
            block: _,
            channel: _,
        } => return None,
    })
}
