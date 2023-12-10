use std::ffi::{c_void, CString};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

// use anyhow::Context;
use ash::vk::{self, Handle};
use bevy::math::uvec2;
use bevy::prelude::*;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use eyre::{Context, ContextCompat};
use openxr as xr;
use wgpu::Instance;
use wgpu_hal::{api::Vulkan as V, Api};
use xr::EnvironmentBlendMode;

use crate::graphics::extensions::XrExtensions;
use crate::input::XrInput;

use crate::resources::{
    OXrSessionSetupInfo, Swapchain, SwapchainInner, VulkanOXrSessionSetupInfo,
    XrEnvironmentBlendMode, XrFormat, XrFrameState, XrFrameWaiter, XrInstance, XrResolution,
    XrSession, XrSessionRunning, XrSwapchain, XrViews,
};
use crate::VIEW_TYPE;

use super::{XrAppInfo, XrPreferdBlendMode};

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

    assert!(available_extensions.raw().khr_vulkan_enable2);
    // info!("available OpenXR extensions: {:#?}", available_extensions);

    let mut enabled_extensions: xr::ExtensionSet =
        (available_extensions & reqeusted_extensions).into();
    enabled_extensions.khr_vulkan_enable2 = true;
    #[cfg(target_os = "android")]
    {
        enabled_extensions.khr_android_create_instance = true;
    }

    let available_layers = xr_entry.enumerate_layers()?;
    // info!("available OpenXR layers: {:#?}", available_layers);

    let xr_instance = xr_entry.create_instance(
        &xr::ApplicationInfo {
            application_name: &app_info.name,
            engine_name: "Bevy",
            ..Default::default()
        },
        &enabled_extensions,
        &[],
    )?;
    info!("created OpenXR instance");
    let instance_props = xr_instance.properties()?;
    let xr_system_id = xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    info!("created OpenXR system");
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

    #[cfg(not(target_os = "android"))]
    let vk_target_version = vk::make_api_version(0, 1, 2, 0);
    #[cfg(not(target_os = "android"))]
    let vk_target_version_xr = xr::Version::new(1, 2, 0);

    #[cfg(target_os = "android")]
    let vk_target_version = vk::make_api_version(0, 1, 1, 0);
    #[cfg(target_os = "android")]
    let vk_target_version_xr = xr::Version::new(1, 1, 0);

    let reqs = xr_instance.graphics_requirements::<xr::Vulkan>(xr_system_id)?;
    if vk_target_version_xr < reqs.min_api_version_supported
        || vk_target_version_xr.major() > reqs.max_api_version_supported.major()
    {
        panic!(
            "OpenXR runtime requires Vulkan version >= {}, < {}.0.0",
            reqs.min_api_version_supported,
            reqs.max_api_version_supported.major() + 1
        );
    }

    let vk_entry = unsafe { ash::Entry::load() }?;
    let flags = wgpu::InstanceFlags::from_build_config();
    let extensions = <V as Api>::Instance::desired_extensions(&vk_entry, vk_target_version, flags)?;
    let device_extensions = vec![
        ash::extensions::khr::Swapchain::name(),
        ash::extensions::khr::DrawIndirectCount::name(),
        #[cfg(target_os = "android")]
        ash::extensions::khr::TimelineSemaphore::name(),
    ];
    info!(
        "creating Vulkan instance with these extensions: {:#?}",
        extensions
    );

    let vk_instance = unsafe {
        let extensions_cchar: Vec<_> = extensions.iter().map(|s| s.as_ptr()).collect();

        let app_name = CString::new(app_info.name)?;
        let vk_app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(1)
            .engine_name(&app_name)
            .engine_version(1)
            .api_version(vk_target_version);

        let vk_instance = xr_instance
            .create_vulkan_instance(
                xr_system_id,
                std::mem::transmute(vk_entry.static_fn().get_instance_proc_addr),
                &vk::InstanceCreateInfo::builder()
                    .application_info(&vk_app_info)
                    .enabled_extension_names(&extensions_cchar) as *const _
                    as *const _,
            )
            .context("OpenXR error creating Vulkan instance")
            .unwrap()
            .map_err(vk::Result::from_raw)
            .context("Vulkan error creating Vulkan instance")
            .unwrap();

        ash::Instance::load(
            vk_entry.static_fn(),
            vk::Instance::from_raw(vk_instance as _),
        )
    };
    info!("created Vulkan instance");

    let vk_instance_ptr = vk_instance.handle().as_raw() as *const c_void;

    let vk_physical_device = vk::PhysicalDevice::from_raw(unsafe {
        xr_instance.vulkan_graphics_device(xr_system_id, vk_instance.handle().as_raw() as _)? as _
    });
    let vk_physical_device_ptr = vk_physical_device.as_raw() as *const c_void;

    let vk_device_properties =
        unsafe { vk_instance.get_physical_device_properties(vk_physical_device) };
    if vk_device_properties.api_version < vk_target_version {
        unsafe { vk_instance.destroy_instance(None) }
        panic!("Vulkan physical device doesn't support version 1.1");
    }

    let wgpu_vk_instance = unsafe {
        <V as Api>::Instance::from_raw(
            vk_entry.clone(),
            vk_instance.clone(),
            vk_target_version,
            0,
            None,
            extensions,
            flags,
            false,
            Some(Box::new(())),
        )?
    };

    let wgpu_features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::MULTIVIEW
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::MULTI_DRAW_INDIRECT;

    let wgpu_exposed_adapter = wgpu_vk_instance
        .expose_adapter(vk_physical_device)
        .context("failed to expose adapter")?;

    let enabled_extensions = wgpu_exposed_adapter
        .adapter
        .required_device_extensions(wgpu_features);

    let (wgpu_open_device, vk_device_ptr, queue_family_index) = {
        let extensions_cchar: Vec<_> = device_extensions.iter().map(|s| s.as_ptr()).collect();
        let mut enabled_phd_features = wgpu_exposed_adapter
            .adapter
            .physical_device_features(&enabled_extensions, wgpu_features);
        let family_index = 0;
        let family_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(family_index)
            .queue_priorities(&[1.0])
            .build();
        let family_infos = [family_info];
        let info = enabled_phd_features
            .add_to_device_create_builder(
                vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&family_infos)
                    .push_next(&mut vk::PhysicalDeviceMultiviewFeatures {
                        multiview: vk::TRUE,
                        ..Default::default()
                    }),
            )
            .enabled_extension_names(&extensions_cchar)
            .build();
        let vk_device = unsafe {
            let vk_device = xr_instance
                .create_vulkan_device(
                    xr_system_id,
                    std::mem::transmute(vk_entry.static_fn().get_instance_proc_addr),
                    vk_physical_device.as_raw() as _,
                    &info as *const _ as *const _,
                )
                .context("OpenXR error creating Vulkan device")?
                .map_err(vk::Result::from_raw)
                .context("Vulkan error creating Vulkan device")?;

            ash::Device::load(vk_instance.fp_v1_0(), vk::Device::from_raw(vk_device as _))
        };
        let vk_device_ptr = vk_device.handle().as_raw() as *const c_void;

        let wgpu_open_device = unsafe {
            wgpu_exposed_adapter.adapter.device_from_raw(
                vk_device,
                true,
                &enabled_extensions,
                wgpu_features,
                family_info.queue_family_index,
                0,
            )
        }?;

        (
            wgpu_open_device,
            vk_device_ptr,
            family_info.queue_family_index,
        )
    };

    let wgpu_instance =
        unsafe { wgpu::Instance::from_hal::<wgpu_hal::api::Vulkan>(wgpu_vk_instance) };
    let wgpu_adapter = unsafe { wgpu_instance.create_adapter_from_hal(wgpu_exposed_adapter) };
    let (wgpu_device, wgpu_queue) = unsafe {
        wgpu_adapter.create_device_from_hal(
            wgpu_open_device,
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu_features,
                required_limits: wgpu::Limits {
                    max_bind_groups: 8,
                    max_storage_buffer_binding_size: wgpu_adapter
                        .limits()
                        .max_storage_buffer_binding_size,
                    max_push_constant_size: 4,
                    ..Default::default()
                },
            },
            None,
        )
    }?;
    Ok((
        xr_instance.into(),
        OXrSessionSetupInfo::Vulkan(VulkanOXrSessionSetupInfo {
            device_ptr: vk_device_ptr,
            physical_device_ptr: vk_physical_device_ptr,
            vk_instance_ptr,
            queue_family_index,
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
        OXrSessionSetupInfo::Vulkan(v) => v,
        _ => eyre::bail!("Wrong Graphics Api"),
    };
    let (session, frame_wait, frame_stream) = unsafe {
        xr_instance.create_session::<xr::Vulkan>(
            xr_instance.system(xr::FormFactor::HEAD_MOUNTED_DISPLAY)?,
            &xr::vulkan::SessionCreateInfo {
                instance: setup_info.vk_instance_ptr,
                physical_device: setup_info.physical_device_ptr,
                device: setup_info.device_ptr,
                queue_family_index: setup_info.queue_family_index,
                queue_index: 0,
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
            format: wgpu_to_vulkan(swapchain_format).as_raw() as _,
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
            let color_image = vk::Image::from_raw(color_image);
            let wgpu_hal_texture = unsafe {
                <V as Api>::Device::texture_from_raw(
                    color_image,
                    &wgpu_hal::TextureDescriptor {
                        label: Some("bevy_openxr swapchain"), // unused internally
                        size: wgpu::Extent3d {
                            width: resolution.x,
                            height: resolution.y,
                            depth_or_array_layers: 2,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: swapchain_format,
                        usage: wgpu_hal::TextureUses::COLOR_TARGET
                            | wgpu_hal::TextureUses::COPY_DST,
                        memory_flags: wgpu_hal::MemoryFlags::empty(),
                        view_formats: vec![],
                    },
                    None,
                )
            };
            let texture = unsafe {
                wgpu_device.create_texture_from_hal::<V>(
                    wgpu_hal_texture,
                    &wgpu::TextureDescriptor {
                        label: Some("bevy_openxr swapchain"),
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
        XrSession::Vulkan(session.clone()),
        resolution.into(),
        swapchain_format.into(),
        // TODO: this shouldn't be in here
        AtomicBool::new(false).into(),
        frame_wait.into(),
        Swapchain::Vulkan(SwapchainInner {
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

fn wgpu_to_vulkan(format: wgpu::TextureFormat) -> vk::Format {
    // Copied with minor modification from:
    // https://github.com/gfx-rs/wgpu/blob/v0.19/wgpu-hal/src/vulkan/conv.rs#L5C1-L153
    // license: MIT OR Apache-2.0
    use ash::vk::Format as F;
    use wgpu::TextureFormat as Tf;
    use wgpu::{AstcBlock, AstcChannel};
    match format {
        Tf::R8Unorm => F::R8_UNORM,
        Tf::R8Snorm => F::R8_SNORM,
        Tf::R8Uint => F::R8_UINT,
        Tf::R8Sint => F::R8_SINT,
        Tf::R16Uint => F::R16_UINT,
        Tf::R16Sint => F::R16_SINT,
        Tf::R16Unorm => F::R16_UNORM,
        Tf::R16Snorm => F::R16_SNORM,
        Tf::R16Float => F::R16_SFLOAT,
        Tf::Rg8Unorm => F::R8G8_UNORM,
        Tf::Rg8Snorm => F::R8G8_SNORM,
        Tf::Rg8Uint => F::R8G8_UINT,
        Tf::Rg8Sint => F::R8G8_SINT,
        Tf::Rg16Unorm => F::R16G16_UNORM,
        Tf::Rg16Snorm => F::R16G16_SNORM,
        Tf::R32Uint => F::R32_UINT,
        Tf::R32Sint => F::R32_SINT,
        Tf::R32Float => F::R32_SFLOAT,
        Tf::Rg16Uint => F::R16G16_UINT,
        Tf::Rg16Sint => F::R16G16_SINT,
        Tf::Rg16Float => F::R16G16_SFLOAT,
        Tf::Rgba8Unorm => F::R8G8B8A8_UNORM,
        Tf::Rgba8UnormSrgb => F::R8G8B8A8_SRGB,
        Tf::Bgra8UnormSrgb => F::B8G8R8A8_SRGB,
        Tf::Rgba8Snorm => F::R8G8B8A8_SNORM,
        Tf::Bgra8Unorm => F::B8G8R8A8_UNORM,
        Tf::Rgba8Uint => F::R8G8B8A8_UINT,
        Tf::Rgba8Sint => F::R8G8B8A8_SINT,
        Tf::Rgb10a2Uint => F::A2B10G10R10_UINT_PACK32,
        Tf::Rgb10a2Unorm => F::A2B10G10R10_UNORM_PACK32,
        Tf::Rg11b10Float => F::B10G11R11_UFLOAT_PACK32,
        Tf::Rg32Uint => F::R32G32_UINT,
        Tf::Rg32Sint => F::R32G32_SINT,
        Tf::Rg32Float => F::R32G32_SFLOAT,
        Tf::Rgba16Uint => F::R16G16B16A16_UINT,
        Tf::Rgba16Sint => F::R16G16B16A16_SINT,
        Tf::Rgba16Unorm => F::R16G16B16A16_UNORM,
        Tf::Rgba16Snorm => F::R16G16B16A16_SNORM,
        Tf::Rgba16Float => F::R16G16B16A16_SFLOAT,
        Tf::Rgba32Uint => F::R32G32B32A32_UINT,
        Tf::Rgba32Sint => F::R32G32B32A32_SINT,
        Tf::Rgba32Float => F::R32G32B32A32_SFLOAT,
        Tf::Depth32Float => F::D32_SFLOAT,
        Tf::Depth32FloatStencil8 => F::D32_SFLOAT_S8_UINT,
        Tf::Depth24Plus | Tf::Depth24PlusStencil8 | Tf::Stencil8 => {
            panic!("Cannot convert format that is dependent on device properties")
        }
        Tf::Depth16Unorm => F::D16_UNORM,
        Tf::NV12 => F::G8_B8R8_2PLANE_420_UNORM,
        Tf::Rgb9e5Ufloat => F::E5B9G9R9_UFLOAT_PACK32,
        Tf::Bc1RgbaUnorm => F::BC1_RGBA_UNORM_BLOCK,
        Tf::Bc1RgbaUnormSrgb => F::BC1_RGBA_SRGB_BLOCK,
        Tf::Bc2RgbaUnorm => F::BC2_UNORM_BLOCK,
        Tf::Bc2RgbaUnormSrgb => F::BC2_SRGB_BLOCK,
        Tf::Bc3RgbaUnorm => F::BC3_UNORM_BLOCK,
        Tf::Bc3RgbaUnormSrgb => F::BC3_SRGB_BLOCK,
        Tf::Bc4RUnorm => F::BC4_UNORM_BLOCK,
        Tf::Bc4RSnorm => F::BC4_SNORM_BLOCK,
        Tf::Bc5RgUnorm => F::BC5_UNORM_BLOCK,
        Tf::Bc5RgSnorm => F::BC5_SNORM_BLOCK,
        Tf::Bc6hRgbUfloat => F::BC6H_UFLOAT_BLOCK,
        Tf::Bc6hRgbFloat => F::BC6H_SFLOAT_BLOCK,
        Tf::Bc7RgbaUnorm => F::BC7_UNORM_BLOCK,
        Tf::Bc7RgbaUnormSrgb => F::BC7_SRGB_BLOCK,
        Tf::Etc2Rgb8Unorm => F::ETC2_R8G8B8_UNORM_BLOCK,
        Tf::Etc2Rgb8UnormSrgb => F::ETC2_R8G8B8_SRGB_BLOCK,
        Tf::Etc2Rgb8A1Unorm => F::ETC2_R8G8B8A1_UNORM_BLOCK,
        Tf::Etc2Rgb8A1UnormSrgb => F::ETC2_R8G8B8A1_SRGB_BLOCK,
        Tf::Etc2Rgba8Unorm => F::ETC2_R8G8B8A8_UNORM_BLOCK,
        Tf::Etc2Rgba8UnormSrgb => F::ETC2_R8G8B8A8_SRGB_BLOCK,
        Tf::EacR11Unorm => F::EAC_R11_UNORM_BLOCK,
        Tf::EacR11Snorm => F::EAC_R11_SNORM_BLOCK,
        Tf::EacRg11Unorm => F::EAC_R11G11_UNORM_BLOCK,
        Tf::EacRg11Snorm => F::EAC_R11G11_SNORM_BLOCK,
        Tf::Astc { block, channel } => match channel {
            AstcChannel::Unorm => match block {
                AstcBlock::B4x4 => F::ASTC_4X4_UNORM_BLOCK,
                AstcBlock::B5x4 => F::ASTC_5X4_UNORM_BLOCK,
                AstcBlock::B5x5 => F::ASTC_5X5_UNORM_BLOCK,
                AstcBlock::B6x5 => F::ASTC_6X5_UNORM_BLOCK,
                AstcBlock::B6x6 => F::ASTC_6X6_UNORM_BLOCK,
                AstcBlock::B8x5 => F::ASTC_8X5_UNORM_BLOCK,
                AstcBlock::B8x6 => F::ASTC_8X6_UNORM_BLOCK,
                AstcBlock::B8x8 => F::ASTC_8X8_UNORM_BLOCK,
                AstcBlock::B10x5 => F::ASTC_10X5_UNORM_BLOCK,
                AstcBlock::B10x6 => F::ASTC_10X6_UNORM_BLOCK,
                AstcBlock::B10x8 => F::ASTC_10X8_UNORM_BLOCK,
                AstcBlock::B10x10 => F::ASTC_10X10_UNORM_BLOCK,
                AstcBlock::B12x10 => F::ASTC_12X10_UNORM_BLOCK,
                AstcBlock::B12x12 => F::ASTC_12X12_UNORM_BLOCK,
            },
            AstcChannel::UnormSrgb => match block {
                AstcBlock::B4x4 => F::ASTC_4X4_SRGB_BLOCK,
                AstcBlock::B5x4 => F::ASTC_5X4_SRGB_BLOCK,
                AstcBlock::B5x5 => F::ASTC_5X5_SRGB_BLOCK,
                AstcBlock::B6x5 => F::ASTC_6X5_SRGB_BLOCK,
                AstcBlock::B6x6 => F::ASTC_6X6_SRGB_BLOCK,
                AstcBlock::B8x5 => F::ASTC_8X5_SRGB_BLOCK,
                AstcBlock::B8x6 => F::ASTC_8X6_SRGB_BLOCK,
                AstcBlock::B8x8 => F::ASTC_8X8_SRGB_BLOCK,
                AstcBlock::B10x5 => F::ASTC_10X5_SRGB_BLOCK,
                AstcBlock::B10x6 => F::ASTC_10X6_SRGB_BLOCK,
                AstcBlock::B10x8 => F::ASTC_10X8_SRGB_BLOCK,
                AstcBlock::B10x10 => F::ASTC_10X10_SRGB_BLOCK,
                AstcBlock::B12x10 => F::ASTC_12X10_SRGB_BLOCK,
                AstcBlock::B12x12 => F::ASTC_12X12_SRGB_BLOCK,
            },
            AstcChannel::Hdr => match block {
                AstcBlock::B4x4 => F::ASTC_4X4_SFLOAT_BLOCK_EXT,
                AstcBlock::B5x4 => F::ASTC_5X4_SFLOAT_BLOCK_EXT,
                AstcBlock::B5x5 => F::ASTC_5X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B6x5 => F::ASTC_6X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B6x6 => F::ASTC_6X6_SFLOAT_BLOCK_EXT,
                AstcBlock::B8x5 => F::ASTC_8X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B8x6 => F::ASTC_8X6_SFLOAT_BLOCK_EXT,
                AstcBlock::B8x8 => F::ASTC_8X8_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x5 => F::ASTC_10X5_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x6 => F::ASTC_10X6_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x8 => F::ASTC_10X8_SFLOAT_BLOCK_EXT,
                AstcBlock::B10x10 => F::ASTC_10X10_SFLOAT_BLOCK_EXT,
                AstcBlock::B12x10 => F::ASTC_12X10_SFLOAT_BLOCK_EXT,
                AstcBlock::B12x12 => F::ASTC_12X12_SFLOAT_BLOCK_EXT,
            },
        },
    }
}
