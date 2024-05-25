use bevy::log::error;
use wgpu_hal::{Adapter, Instance};
use winapi::shared::dxgiformat::DXGI_FORMAT;
use winapi::um::d3d12 as winapi_d3d12;

use super::{GraphicsExt, GraphicsType, GraphicsWrap};
use crate::error::OxrError;
use crate::types::{AppInfo, OxrExtensions, Result, WgpuGraphics};

unsafe impl GraphicsExt for openxr::D3D12 {
    fn wrap<T: GraphicsType>(item: T::Inner<Self>) -> GraphicsWrap<T> {
        GraphicsWrap::D3D12(item)
    }

    fn required_exts() -> OxrExtensions {
        let mut extensions = openxr::ExtensionSet::default();
        extensions.khr_d3d12_enable = true;
        extensions.into()
    }

    fn from_wgpu_format(format: wgpu::TextureFormat) -> Option<Self::Format> {
        wgpu_to_d3d12(format)
    }

    fn into_wgpu_format(format: Self::Format) -> Option<wgpu::TextureFormat> {
        d3d12_to_wgpu(format)
    }

    unsafe fn to_wgpu_img(
        image: Self::SwapchainImage,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        resolution: bevy::prelude::UVec2,
    ) -> Result<wgpu::Texture> {
        let wgpu_hal_texture = <wgpu_hal::dx12::Api as wgpu_hal::Api>::Device::texture_from_raw(
            d3d12::ComPtr::from_raw(image as *mut _),
            format,
            wgpu::TextureDimension::D2,
            wgpu::Extent3d {
                width: resolution.x,
                height: resolution.y,
                depth_or_array_layers: 2,
            },
            1,
            1,
        );
        let texture = device.create_texture_from_hal::<wgpu_hal::dx12::Api>(
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
                format: format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
        );
        Ok(texture)
    }

    fn init_graphics(
        app_info: &AppInfo,
        instance: &openxr::Instance,
        system_id: openxr::SystemId,
    ) -> Result<(WgpuGraphics, Self::SessionCreateInfo)> {
        let reqs = instance.graphics_requirements::<openxr::D3D12>(system_id)?;

        let instance_descriptor = &wgpu_hal::InstanceDescriptor {
            name: &app_info.name,
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or(
                wgpu::Dx12Compiler::Dxc {
                    dxil_path: None,
                    dxc_path: None,
                },
            ),
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
            .ok_or(OxrError::InitError(
                crate::error::InitError::FailedToFindD3D12Adapter,
            ))?;

        let wgpu_instance =
            unsafe { wgpu::Instance::from_hal::<wgpu_hal::api::Dx12>(wgpu_raw_instance) };

        let wgpu_features = wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
            | wgpu::Features::MULTIVIEW
            | wgpu::Features::MULTI_DRAW_INDIRECT_COUNT
            | wgpu::Features::MULTI_DRAW_INDIRECT;

        let wgpu_limits = wgpu_exposed_adapter.capabilities.limits.clone();

        let wgpu_open_device = unsafe {
            wgpu_exposed_adapter
                .adapter
                .open(wgpu_features, &wgpu_limits)?
        };

        let device_supported_feature_level: d3d12::FeatureLevel =
            get_device_feature_level(wgpu_open_device.device.raw_device());

        if (device_supported_feature_level as u32) < (reqs.min_feature_level as u32) {
            error!(
                "OpenXR runtime requires D3D12 feature level >= {}",
                reqs.min_feature_level
            );
            return Err(OxrError::FailedGraphicsRequirements);
        }

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
            WgpuGraphics(
                wgpu_device,
                wgpu_queue,
                wgpu_adapter.get_info(),
                wgpu_adapter,
                wgpu_instance,
            ),
            Self::SessionCreateInfo {
                device: raw_device.cast(),
                queue: raw_queue.cast(),
            },
        ))
    }

    unsafe fn create_session(
        instance: &openxr::Instance,
        system_id: openxr::SystemId,
        info: &Self::SessionCreateInfo,
        session_create_info_chain: &mut OxrSessionCreateInfoChain,
    ) -> openxr::Result<(
        openxr::Session<Self>,
        openxr::FrameWaiter,
        openxr::FrameStream<Self>,
    )> {
        let binding = sys::GraphicsBindingD3D12KHR {
            ty: sys::GraphicsBindingD3D12KHR::TYPE,
            next: session_create_info_chain.chain_pointer(),
            device: info.device,
            queue: info.queue,
        };
        let info = sys::SessionCreateInfo {
            ty: sys::SessionCreateInfo::TYPE,
            next: &binding as *const _ as *const _,
            create_flags: Default::default(),
            system_id: system,
        };
        let mut out = sys::Session::NULL;
        cvt((instance.fp().create_session)(
            instance.as_raw(),
            &info,
            &mut out,
        ))?;
        Ok(openxr::Session::from_raw(
            instance.clone(),
            out,
            Box::new(()),
        ))
    }
}

// Extracted from https://github.com/gfx-rs/wgpu/blob/1161a22f4fbb4fc204eb06f2ac4243f83e0e980d/wgpu-hal/src/dx12/adapter.rs#L73-L94
// license: MIT OR Apache-2.0
fn get_device_feature_level(
    device: &d3d12::ComPtr<winapi_d3d12::ID3D12Device>,
) -> d3d12::FeatureLevel {
    // Detect the highest supported feature level.
    let d3d_feature_level = [
        d3d12::FeatureLevel::L12_1,
        d3d12::FeatureLevel::L12_0,
        d3d12::FeatureLevel::L11_1,
        d3d12::FeatureLevel::L11_0,
    ];
    type FeatureLevelsInfo = winapi_d3d12::D3D12_FEATURE_DATA_FEATURE_LEVELS;
    let mut device_levels: FeatureLevelsInfo = unsafe { std::mem::zeroed() };
    device_levels.NumFeatureLevels = d3d_feature_level.len() as u32;
    device_levels.pFeatureLevelsRequested = d3d_feature_level.as_ptr().cast();
    unsafe {
        device.CheckFeatureSupport(
            winapi_d3d12::D3D12_FEATURE_FEATURE_LEVELS,
            (&mut device_levels as *mut FeatureLevelsInfo).cast(),
            std::mem::size_of::<FeatureLevelsInfo>() as _,
        )
    };
    // This cast should never fail because we only requested feature levels that are already in the enum.
    let max_feature_level = d3d12::FeatureLevel::try_from(device_levels.MaxSupportedFeatureLevel)
        .expect("Unexpected feature level");
    max_feature_level
}

fn d3d12_to_wgpu(format: DXGI_FORMAT) -> Option<wgpu::TextureFormat> {
    use wgpu::TextureFormat as Tf;
    use winapi::shared::dxgiformat::*;

    Some(match format {
        DXGI_FORMAT_R8_UNORM => Tf::R8Unorm,
        DXGI_FORMAT_R8_SNORM => Tf::R8Snorm,
        DXGI_FORMAT_R8_UINT => Tf::R8Uint,
        DXGI_FORMAT_R8_SINT => Tf::R8Sint,
        DXGI_FORMAT_R16_UINT => Tf::R16Uint,
        DXGI_FORMAT_R16_SINT => Tf::R16Sint,
        DXGI_FORMAT_R16_UNORM => Tf::R16Unorm,
        DXGI_FORMAT_R16_SNORM => Tf::R16Snorm,
        DXGI_FORMAT_R16_FLOAT => Tf::R16Float,
        DXGI_FORMAT_R8G8_UNORM => Tf::Rg8Unorm,
        DXGI_FORMAT_R8G8_SNORM => Tf::Rg8Snorm,
        DXGI_FORMAT_R8G8_UINT => Tf::Rg8Uint,
        DXGI_FORMAT_R8G8_SINT => Tf::Rg8Sint,
        DXGI_FORMAT_R16G16_UNORM => Tf::Rg16Unorm,
        DXGI_FORMAT_R16G16_SNORM => Tf::Rg16Snorm,
        DXGI_FORMAT_R32_UINT => Tf::R32Uint,
        DXGI_FORMAT_R32_SINT => Tf::R32Sint,
        DXGI_FORMAT_R32_FLOAT => Tf::R32Float,
        DXGI_FORMAT_R16G16_UINT => Tf::Rg16Uint,
        DXGI_FORMAT_R16G16_SINT => Tf::Rg16Sint,
        DXGI_FORMAT_R16G16_FLOAT => Tf::Rg16Float,
        DXGI_FORMAT_R8G8B8A8_UNORM => Tf::Rgba8Unorm,
        DXGI_FORMAT_R8G8B8A8_UNORM_SRGB => Tf::Rgba8UnormSrgb,
        DXGI_FORMAT_B8G8R8A8_UNORM_SRGB => Tf::Bgra8UnormSrgb,
        DXGI_FORMAT_R8G8B8A8_SNORM => Tf::Rgba8Snorm,
        DXGI_FORMAT_B8G8R8A8_UNORM => Tf::Bgra8Unorm,
        DXGI_FORMAT_R8G8B8A8_UINT => Tf::Rgba8Uint,
        DXGI_FORMAT_R8G8B8A8_SINT => Tf::Rgba8Sint,
        DXGI_FORMAT_R9G9B9E5_SHAREDEXP => Tf::Rgb9e5Ufloat,
        DXGI_FORMAT_R10G10B10A2_UINT => Tf::Rgb10a2Uint,
        DXGI_FORMAT_R10G10B10A2_UNORM => Tf::Rgb10a2Unorm,
        DXGI_FORMAT_R11G11B10_FLOAT => Tf::Rg11b10Float,
        DXGI_FORMAT_R32G32_UINT => Tf::Rg32Uint,
        DXGI_FORMAT_R32G32_SINT => Tf::Rg32Sint,
        DXGI_FORMAT_R32G32_FLOAT => Tf::Rg32Float,
        DXGI_FORMAT_R16G16B16A16_UINT => Tf::Rgba16Uint,
        DXGI_FORMAT_R16G16B16A16_SINT => Tf::Rgba16Sint,
        DXGI_FORMAT_R16G16B16A16_UNORM => Tf::Rgba16Unorm,
        DXGI_FORMAT_R16G16B16A16_SNORM => Tf::Rgba16Snorm,
        DXGI_FORMAT_R16G16B16A16_FLOAT => Tf::Rgba16Float,
        DXGI_FORMAT_R32G32B32A32_UINT => Tf::Rgba32Uint,
        DXGI_FORMAT_R32G32B32A32_SINT => Tf::Rgba32Sint,
        DXGI_FORMAT_R32G32B32A32_FLOAT => Tf::Rgba32Float,
        DXGI_FORMAT_D24_UNORM_S8_UINT => Tf::Stencil8,
        DXGI_FORMAT_D16_UNORM => Tf::Depth16Unorm,
        DXGI_FORMAT_D32_FLOAT => Tf::Depth32Float,
        DXGI_FORMAT_D32_FLOAT_S8X24_UINT => Tf::Depth32FloatStencil8,
        DXGI_FORMAT_NV12 => Tf::NV12,
        DXGI_FORMAT_BC1_UNORM => Tf::Bc1RgbaUnorm,
        DXGI_FORMAT_BC1_UNORM_SRGB => Tf::Bc1RgbaUnormSrgb,
        DXGI_FORMAT_BC2_UNORM => Tf::Bc2RgbaUnorm,
        DXGI_FORMAT_BC2_UNORM_SRGB => Tf::Bc2RgbaUnormSrgb,
        DXGI_FORMAT_BC3_UNORM => Tf::Bc3RgbaUnorm,
        DXGI_FORMAT_BC3_UNORM_SRGB => Tf::Bc3RgbaUnormSrgb,
        DXGI_FORMAT_BC4_UNORM => Tf::Bc4RUnorm,
        DXGI_FORMAT_BC4_SNORM => Tf::Bc4RSnorm,
        DXGI_FORMAT_BC5_UNORM => Tf::Bc5RgUnorm,
        DXGI_FORMAT_BC5_SNORM => Tf::Bc5RgSnorm,
        DXGI_FORMAT_BC6H_UF16 => Tf::Bc6hRgbUfloat,
        DXGI_FORMAT_BC6H_SF16 => Tf::Bc6hRgbFloat,
        DXGI_FORMAT_BC7_UNORM => Tf::Bc7RgbaUnorm,
        DXGI_FORMAT_BC7_UNORM_SRGB => Tf::Bc7RgbaUnormSrgb,
        _ => return None,
    })
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
