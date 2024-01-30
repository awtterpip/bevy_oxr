use std::ffi::{c_void, CString};
use std::sync::Mutex;

use super::{Swapchain, SwapchainInner, VIEW_TYPE};
use crate::backend::oxr::{OXrInstance, OXrSession};
use crate::error::{Result, XrError};
use ash::vk::{self, Handle};
use glam::uvec2;

pub fn init_oxr_graphics(
    instance: OXrInstance,
    swapchain_format: wgpu::TextureFormat,
    xr_app_name: String,
) -> Result<OXrSession> {
    use wgpu_hal::{api::Vulkan as V, Api};
    let xr_instance = &instance.0;
    let xr_system_id = xr_instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    // TODO! allow selecting specific blend mode.
    let blend_mode = openxr::EnvironmentBlendMode::OPAQUE;

    #[cfg(not(target_os = "android"))]
    let vk_target_version = vk::make_api_version(0, 1, 2, 0);
    #[cfg(not(target_os = "android"))]
    let vk_target_version_xr = openxr::Version::new(1, 2, 0);

    #[cfg(target_os = "android")]
    let vk_target_version = vk::make_api_version(0, 1, 1, 0);
    #[cfg(target_os = "android")]
    let vk_target_version_xr = openxr::Version::new(1, 1, 0);

    let reqs = xr_instance.graphics_requirements::<openxr::Vulkan>(xr_system_id)?;
    if vk_target_version_xr < reqs.min_api_version_supported
        || vk_target_version_xr.major() > reqs.max_api_version_supported.major()
    {
        return Err(XrError::Placeholder);
    }

    let vk_entry = unsafe { ash::Entry::load() }.map_err(|_| XrError::Placeholder)?;
    let flags = wgpu_hal::InstanceFlags::empty();
    let extensions = <V as Api>::Instance::required_extensions(&vk_entry, vk_target_version, flags)
        .map_err(|_| XrError::Placeholder)?;
    let device_extensions = vec![
        ash::extensions::khr::Swapchain::name(),
        ash::extensions::khr::DrawIndirectCount::name(),
        #[cfg(target_os = "android")]
        ash::extensions::khr::TimelineSemaphore::name(),
    ];

    let vk_instance = unsafe {
        let extensions_cchar: Vec<_> = extensions.iter().map(|s| s.as_ptr()).collect();

        let app_name = CString::new(xr_app_name).map_err(|_| XrError::Placeholder)?;
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
            .unwrap()
            .map_err(vk::Result::from_raw)
            .unwrap();

        ash::Instance::load(
            vk_entry.static_fn(),
            vk::Instance::from_raw(vk_instance as _),
        )
    };

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
        )
        .map_err(|_| XrError::Placeholder)?
    };

    let wgpu_features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu::Features::MULTIVIEW
        | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
        | wgpu::Features::MULTI_DRAW_INDIRECT;

    let wgpu_exposed_adapter = wgpu_vk_instance
        .expose_adapter(vk_physical_device)
        .ok_or(XrError::Placeholder)?;

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
                )?
                .map_err(|_| XrError::Placeholder)?;

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
        }
        .map_err(|_| XrError::Placeholder)?;

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
    }
    .map_err(|_| XrError::Placeholder)?;

    let (session, frame_wait, frame_stream) = unsafe {
        xr_instance.create_session::<openxr::Vulkan>(
            xr_system_id,
            &openxr::vulkan::SessionCreateInfo {
                instance: vk_instance_ptr,
                physical_device: vk_physical_device_ptr,
                device: vk_device_ptr,
                queue_family_index,
                queue_index: 0,
            },
        )
    }?;

    let views = xr_instance.enumerate_view_configuration_views(xr_system_id, VIEW_TYPE)?;

    let resolution = uvec2(
        views[0].recommended_image_rect_width,
        views[0].recommended_image_rect_height,
    );

    let swapchain = session
        .create_swapchain(&openxr::SwapchainCreateInfo {
            create_flags: openxr::SwapchainCreateFlags::EMPTY,
            usage_flags: openxr::SwapchainUsageFlags::COLOR_ATTACHMENT
                | openxr::SwapchainUsageFlags::SAMPLED,
            format: wgpu_to_vulkan(swapchain_format)
                .ok_or(XrError::Placeholder)?
                .as_raw() as _,
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
    let images = swapchain.enumerate_images().unwrap();

    let buffers: Vec<_> = images
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

    Ok(OXrSession {
        inner_instance: instance.0.clone(),
        instance: instance.into(),
        session: session.clone().into_any_graphics(),
        render_resources: Mutex::new(Some((
            wgpu_device,
            wgpu_queue,
            wgpu_adapter.get_info(),
            wgpu_adapter,
            wgpu_instance,
        ))),
        resolution,
        views: Mutex::new([openxr::View::default(), openxr::View::default()]),
        swapchain: Swapchain::Vulkan(SwapchainInner {
            stream: Mutex::new(frame_stream),
            swapchain: Mutex::new(swapchain),
            buffers,
            image_index: Mutex::new(0),
        }),
        frame_state: Mutex::new(openxr::FrameState {
            predicted_display_time: openxr::Time::from_nanos(1),
            predicted_display_period: openxr::Duration::from_nanos(1),
            should_render: true,
        }),
        blend_mode,
        frame_waiter: Mutex::new(frame_wait),
        stage: session
            .create_reference_space(openxr::ReferenceSpaceType::STAGE, openxr::Posef::IDENTITY)?,
        head: session
            .create_reference_space(openxr::ReferenceSpaceType::VIEW, openxr::Posef::IDENTITY)?,
        format: swapchain_format,
    })
}

fn wgpu_to_vulkan(format: wgpu::TextureFormat) -> Option<vk::Format> {
    // Copied with minor modification from:
    // https://github.com/gfx-rs/wgpu/blob/a7defb723f856d946d6d220e9897d20dbb7b8f61/wgpu-hal/src/vulkan/conv.rs#L5-L151
    // license: MIT OR Apache-2.0
    use ash::vk::Format as F;
    use wgpu::TextureFormat as Tf;
    use wgpu::{AstcBlock, AstcChannel};
    Some(match format {
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
        //        Tf::Rgb10a2Uint => F::A2B10G10R10_UINT_PACK32,
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
        Tf::Depth24Plus | Tf::Depth24PlusStencil8 | Tf::Stencil8 => return None, // Dependent on device properties
        Tf::Depth16Unorm => F::D16_UNORM,
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
    })
}
