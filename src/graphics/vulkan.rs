use std::ffi::{c_void, CString};
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

use anyhow::Context;
use ash::vk::{self, Handle};
use bevy::math::uvec2;
use bevy::prelude::*;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use openxr as xr;
use wgpu::{Instance, Texture};

use crate::input::XrInput;
use crate::resources::{
    XrEnvironmentBlendMode, XrFrameWaiter, XrInstance, XrSession, XrSessionRunning, XrSwapchain, Swapchain, SwapchainInner, XrViews, XrFrameState,
};
use crate::VIEW_TYPE;

pub fn initialize_xr_graphics(window: Option<RawHandleWrapper>) -> anyhow::Result<(
    RenderDevice,
    RenderQueue,
    RenderAdapterInfo,
    RenderAdapter,
    Instance,
    XrInstance,
    XrSession,
    XrEnvironmentBlendMode,
    XrSessionRunning,
    XrFrameWaiter,
    XrSwapchain,
    XrInput,
    XrViews,
    XrFrameState,
)> {
    use wgpu_hal::{api::Vulkan as V, Api};

    let xr_entry = unsafe { xr::Entry::load() }?;

    let available_extensions = xr_entry.enumerate_extensions()?;
    assert!(available_extensions.khr_vulkan_enable2);
    info!("available xr exts: {:#?}", available_extensions);

    let mut enabled_extensions = xr::ExtensionSet::default();
    enabled_extensions.khr_vulkan_enable2 = true;
    #[cfg(target_os = "android")]
    {
        enabled_extensions.khr_android_create_instance = true;
    }

    let available_layers = xr_entry.enumerate_layers()?;
    info!("available xr layers: {:#?}", available_layers);

    let xr_instance = xr_entry.create_instance(
        &xr::ApplicationInfo {
            application_name: "Ambient",
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

    let blend_mode = xr_instance.enumerate_environment_blend_modes(xr_system_id, VIEW_TYPE)?[0];

    let vk_target_version = vk::make_api_version(0, 1, 2, 0);
    let vk_target_version_xr = xr::Version::new(1, 2, 0);
    let reqs = xr_instance.graphics_requirements::<xr::Vulkan>(xr_system_id)?;
    if vk_target_version_xr < reqs.min_api_version_supported
        || vk_target_version_xr.major() > reqs.max_api_version_supported.major()
    {
        panic!(
            "OpenXR runtime requires Vulkan version > {}, < {}.0.0",
            reqs.min_api_version_supported,
            reqs.max_api_version_supported.major() + 1
        );
    }

    let vk_entry = unsafe { ash::Entry::load() }?;
    let flags = wgpu_hal::InstanceFlags::empty();
    let extensions =
        <V as Api>::Instance::required_extensions(&vk_entry, vk_target_version, flags)?;
    let device_extensions = vec![
        ash::extensions::khr::Swapchain::name(),
        ash::extensions::khr::DrawIndirectCount::name(),
    ];
    info!(
        "creating vulkan instance with these extensions: {:#?}",
        extensions
    );

    let vk_instance = unsafe {
        let extensions_cchar: Vec<_> = extensions.iter().map(|s| s.as_ptr()).collect();

        let app_name = CString::new("Ambient")?;
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
            .context("XR error creating Vulkan instance")
            .unwrap()
            .map_err(vk::Result::from_raw)
            .context("Vulkan error creating Vulkan instance")
            .unwrap();

        ash::Instance::load(
            vk_entry.static_fn(),
            vk::Instance::from_raw(vk_instance as _),
        )
    };
    info!("created vulkan instance");

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
                .context("XR error creating Vulkan device")?
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
                features: wgpu_features,
                limits: wgpu::Limits {
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

    let (session, frame_wait, frame_stream) = unsafe {
        xr_instance.create_session::<xr::Vulkan>(
            xr_system_id,
            &xr::vulkan::SessionCreateInfo {
                instance: vk_instance_ptr,
                physical_device: vk_physical_device_ptr,
                device: vk_device_ptr,
                queue_family_index,
                queue_index: 0,
            },
        )
    }?;

    let views = xr_instance.enumerate_view_configuration_views(xr_system_id, VIEW_TYPE)?;

    let surface = window.map(|wrapper| unsafe {
        // SAFETY: Plugins should be set up on the main thread.
        let handle = wrapper.get_handle();
        wgpu_instance
            .create_surface(&handle)
            .expect("Failed to create wgpu surface")
    });
    let swapchain_format = surface
            .as_ref()
            .map(|surface| surface.get_capabilities(&wgpu_adapter).formats[0])
            .unwrap_or(wgpu::TextureFormat::Rgba8UnormSrgb);

    let resolution = uvec2(views[0].recommended_image_rect_width, views[0].recommended_image_rect_height);

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
                let color_image = vk::Image::from_raw(color_image);
                let wgpu_hal_texture = unsafe {
                    <V as Api>::Device::texture_from_raw(
                        color_image,
                        &wgpu_hal::TextureDescriptor {
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
        wgpu_device.into(),
        RenderQueue(Arc::new(wgpu_queue)),
        RenderAdapterInfo(wgpu_adapter.get_info()),
        RenderAdapter(Arc::new(wgpu_adapter)),
        wgpu_instance,
        xr_instance.clone().into(),
        session.clone().into_any_graphics().into(),
        blend_mode.into(),
        AtomicBool::new(false).into(),
        Mutex::new(frame_wait).into(),
        Mutex::new(Swapchain::Vulkan(SwapchainInner {
            stream: frame_stream,
            handle,
            resolution,
            format: swapchain_format,
            buffers,
            image_index: 0,
        })).into(),
        XrInput::new(xr_instance, session.into_any_graphics())?,
        Mutex::default().into(),
        Mutex::default().into(),
    ))
}

fn wgpu_to_vulkan(format: wgpu::TextureFormat) -> vk::Format {
    use vk::Format;
    match format {
        wgpu::TextureFormat::R8Unorm => Format::R8_UNORM,
        wgpu::TextureFormat::R8Snorm => Format::R8_SNORM,
        wgpu::TextureFormat::R8Uint => Format::R8_UINT,
        wgpu::TextureFormat::R8Sint => Format::R8_SINT,
        wgpu::TextureFormat::R16Uint => Format::R16_UINT,
        wgpu::TextureFormat::R16Sint => Format::R16_SINT,
        wgpu::TextureFormat::R16Unorm => Format::R16_UNORM,
        wgpu::TextureFormat::R16Snorm => Format::R16_SNORM,
        wgpu::TextureFormat::R16Float => Format::R16_SFLOAT,
        wgpu::TextureFormat::Rg8Unorm => Format::R8G8_UNORM,
        wgpu::TextureFormat::Rg8Snorm => Format::R8G8_SNORM,
        wgpu::TextureFormat::Rg8Uint => Format::R8G8_UINT,
        wgpu::TextureFormat::Rg8Sint => Format::R8G8_SINT,
        wgpu::TextureFormat::R32Uint => Format::R32_UINT,
        wgpu::TextureFormat::R32Sint => Format::R32_SINT,
        wgpu::TextureFormat::R32Float => Format::R32_SFLOAT,
        wgpu::TextureFormat::Rg16Uint => Format::R16G16_UINT,
        wgpu::TextureFormat::Rg16Sint => Format::R16G16_SINT,
        wgpu::TextureFormat::Rg16Unorm => Format::R16G16_UNORM,
        wgpu::TextureFormat::Rg16Snorm => Format::R16G16_SNORM,
        wgpu::TextureFormat::Rg16Float => Format::R16G16_SFLOAT,
        wgpu::TextureFormat::Rgba8Unorm => Format::R8G8B8A8_UNORM,
        wgpu::TextureFormat::Rgba8UnormSrgb => Format::R8G8B8A8_SRGB,
        wgpu::TextureFormat::Rgba8Snorm => Format::R8G8B8A8_SNORM,
        wgpu::TextureFormat::Rgba8Uint => Format::R8G8B8A8_UINT,
        wgpu::TextureFormat::Rgba8Sint => Format::R8G8B8A8_SINT,
        wgpu::TextureFormat::Bgra8Unorm => Format::B8G8R8A8_UNORM,
        wgpu::TextureFormat::Bgra8UnormSrgb => Format::B8G8R8A8_SRGB,
        wgpu::TextureFormat::Rgb9e5Ufloat => Format::E5B9G9R9_UFLOAT_PACK32, // this might be the wrong type??? i can't tell
        wgpu::TextureFormat::Rgb10a2Unorm => Format::A2R10G10B10_UNORM_PACK32,
        wgpu::TextureFormat::Rg11b10Float => panic!("this texture type invokes nothing but fear within my soul and i don't think vulkan has a proper type for this"),
        wgpu::TextureFormat::Rg32Uint => Format::R32G32_UINT,
        wgpu::TextureFormat::Rg32Sint => Format::R32G32_SINT,
        wgpu::TextureFormat::Rg32Float => Format::R32G32_SFLOAT,
        wgpu::TextureFormat::Rgba16Uint => Format::R16G16B16A16_UINT,
        wgpu::TextureFormat::Rgba16Sint => Format::R16G16B16A16_SINT,
        wgpu::TextureFormat::Rgba16Unorm => Format::R16G16B16A16_UNORM,
        wgpu::TextureFormat::Rgba16Snorm => Format::R16G16B16A16_SNORM,
        wgpu::TextureFormat::Rgba16Float => Format::R16G16B16A16_SFLOAT,
        wgpu::TextureFormat::Rgba32Uint => Format::R32G32B32A32_UINT,
        wgpu::TextureFormat::Rgba32Sint => Format::R32G32B32A32_SINT,
        wgpu::TextureFormat::Rgba32Float => Format::R32G32B32A32_SFLOAT,
        wgpu::TextureFormat::Stencil8 => Format::S8_UINT,
        wgpu::TextureFormat::Depth16Unorm => Format::D16_UNORM,
        wgpu::TextureFormat::Depth24Plus => Format::X8_D24_UNORM_PACK32,
        wgpu::TextureFormat::Depth24PlusStencil8 => Format::D24_UNORM_S8_UINT,
        wgpu::TextureFormat::Depth32Float => Format::D32_SFLOAT,
        wgpu::TextureFormat::Depth32FloatStencil8 => Format::D32_SFLOAT_S8_UINT,
        wgpu::TextureFormat::Etc2Rgb8Unorm => Format::ETC2_R8G8B8_UNORM_BLOCK,
        wgpu::TextureFormat::Etc2Rgb8UnormSrgb => Format::ETC2_R8G8B8_SRGB_BLOCK,
        wgpu::TextureFormat::Etc2Rgb8A1Unorm => Format::ETC2_R8G8B8A1_UNORM_BLOCK,
        wgpu::TextureFormat::Etc2Rgb8A1UnormSrgb => Format::ETC2_R8G8B8A1_SRGB_BLOCK,
        wgpu::TextureFormat::Etc2Rgba8Unorm => Format::ETC2_R8G8B8A8_UNORM_BLOCK,
        wgpu::TextureFormat::Etc2Rgba8UnormSrgb => Format::ETC2_R8G8B8A8_SRGB_BLOCK,
        wgpu::TextureFormat::EacR11Unorm => Format::EAC_R11_UNORM_BLOCK,
        wgpu::TextureFormat::EacR11Snorm => Format::EAC_R11_SNORM_BLOCK,
        wgpu::TextureFormat::EacRg11Unorm => Format::EAC_R11G11_UNORM_BLOCK,
        wgpu::TextureFormat::EacRg11Snorm => Format::EAC_R11G11_SNORM_BLOCK,
        wgpu::TextureFormat::Astc { .. } => panic!("please god kill me now"),
        _ => panic!("fuck no")
    }
}