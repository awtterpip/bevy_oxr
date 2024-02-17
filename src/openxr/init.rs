pub mod vulkan;

use bevy::log::{info, warn};
use bevy::math::{uvec2, UVec2};
use thiserror::Error;

use crate::openxr::resources::*;
use crate::types::BlendMode;

use super::extensions::XrExtensions;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Version(pub u8, pub u8, pub u16);

impl Version {
    pub const BEVY: Self = Self(0, 12, 1);

    pub const fn to_u32(self) -> u32 {
        let major = (self.0 as u32) << 24;
        let minor = (self.1 as u32) << 16;
        self.2 as u32 | major | minor
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AppInfo<'a> {
    pub name: &'a str,
    pub version: Version,
}

#[derive(Clone, Copy, Debug)]
pub enum GraphicsBackend {
    Vulkan,
}

impl GraphicsBackend {
    const ALL: &'static [Self] = &[Self::Vulkan];

    pub fn available_backends(exts: &XrExtensions) -> Vec<Self> {
        Self::ALL
            .iter()
            .copied()
            .filter(|backend| backend.is_available(exts))
            .collect()
    }

    pub fn is_available(&self, exts: &XrExtensions) -> bool {
        self.required_exts().is_available(exts)
    }

    pub fn required_exts(&self) -> XrExtensions {
        match self {
            GraphicsBackend::Vulkan => vulkan::required_exts(),
        }
    }
}

#[derive(Error, Debug)]
pub enum XrInitError {
    #[error("OpenXR error: {0}")]
    OpenXrError(#[from] openxr::sys::Result),
    #[error("OpenXR loading error: {0}")]
    OpenXrLoadingError(#[from] openxr::LoadError),
    #[error("WGPU instance error: {0}")]
    WgpuInstanceError(#[from] wgpu_hal::InstanceError),
    #[error("WGPU device error: {0}")]
    WgpuDeviceError(#[from] wgpu_hal::DeviceError),
    #[error("WGPU request device error: {0}")]
    WgpuRequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Unsupported texture format: {0:?}")]
    UnsupportedTextureFormat(wgpu::TextureFormat),
    #[error("Vulkan error: {0}")]
    VulkanError(#[from] ash::vk::Result),
    #[error("Vulkan loading error: {0}")]
    VulkanLoadingError(#[from] ash::LoadingError),
    #[error("Graphics backend '{0:?}' is not available")]
    UnavailableBackend(GraphicsBackend),
    #[error("Could not meet graphics requirements for platform. See console for details")]
    FailedGraphicsRequirements,
    #[error("Failed to create CString: {0}")]
    NulError(#[from] std::ffi::NulError),
}

pub trait GraphicsExt: openxr::Graphics {
    fn from_wgpu_format(format: wgpu::TextureFormat) -> Option<Self::Format>;
    fn to_wgpu_format(format: Self::Format) -> Option<wgpu::TextureFormat>;
    fn create_session(
        app_info: AppInfo,
        instance: &openxr::Instance,
        system_id: openxr::SystemId,
        format: wgpu::TextureFormat,
        resolution: UVec2,
    ) -> Result<
        (
            wgpu::Device,
            wgpu::Queue,
            wgpu::Adapter,
            wgpu::Instance,
            openxr::Session<openxr::AnyGraphics>,
            openxr::FrameWaiter,
            FrameStreamInner,
        ),
        XrInitError,
    >;
}

fn xr_entry() -> Result<openxr::Entry, XrInitError> {
    #[cfg(windows)]
    let entry = Some(openxr::Entry::linked());
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(entry)
}

// pub fn init_xr(
//     app_info: AppInfo,
//     requested_exts: XrExtensions,
//     format: wgpu::TextureFormat,
//     preferred_blend_mode: BlendMode,
// ) -> Result<
//     (
//         wgpu::Device,
//         wgpu::Queue,
//         wgpu::Adapter,
//         wgpu::Instance,
//         openxr::Instance,
//         openxr::Session<openxr::AnyGraphics>,
//         openxr::FrameWaiter,
//         FrameStreamInner,
//         XrSwapchain,
//     ),
//     XrInitError,
// > {
//     let entry = xr_entry().unwrap();

//     let required_exts = vulkan::required_exts() | requested_exts;
//     let available_exts: XrExtensions = entry.enumerate_extensions()?.into();
//     for ext in available_exts.unavailable_exts(&required_exts) {
//         warn!("OpenXR extension '{ext}' is not supported by the current OpenXR runtime")
//     }
//     let enabled_exts = required_exts & available_exts;

//     let instance = entry.create_instance(
//         &openxr::ApplicationInfo {
//             application_name: app_info.name,
//             application_version: app_info.version.to_u32(),
//             engine_name: "Bevy",
//             engine_version: Version::BEVY.to_u32(),
//         },
//         &enabled_exts.into(),
//         &[],
//     )?;
//     info!("Created OpenXR Instance: {:#?}", instance.properties()?);

//     let system_id = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
//     info!(
//         "Using system: {:#?}",
//         instance.system_properties(system_id)?
//     );

//     let view = instance
//         .enumerate_view_configurations(system_id)?
//         .first()
//         .copied()
//         .unwrap_or(openxr::ViewConfigurationType::PRIMARY_STEREO);

//     let resolution = instance
//         .enumerate_view_configuration_views(system_id, view)?
//         .first()
//         .map(|c| {
//             uvec2(
//                 c.recommended_image_rect_width,
//                 c.recommended_image_rect_height,
//             )
//         })
//         .unwrap();

//     let (wgpu_device, wgpu_queue, wgpu_adapter, wgpu_instance, session, frame_waiter, frame_stream) =
//         vulkan::create_session(app_info, &instance, system_id, format, resolution)?;

//     let blend_modes = instance.enumerate_environment_blend_modes(system_id, view)?;

//     let blend_mode = if blend_modes.contains(&preferred_blend_mode.into()) {
//         preferred_blend_mode.into()
//     } else {
//         warn!(
//             "Runtime does not support blend mode '{:?}'",
//             preferred_blend_mode
//         );
//         blend_modes
//             .first()
//             .copied()
//             .unwrap_or(openxr::EnvironmentBlendMode::OPAQUE)
//     };
//     info!("Using blend mode '{:?}'", blend_mode);

//     todo!()
// }

impl From<openxr::EnvironmentBlendMode> for BlendMode {
    fn from(value: openxr::EnvironmentBlendMode) -> Self {
        use openxr::EnvironmentBlendMode;
        if value == EnvironmentBlendMode::OPAQUE {
            BlendMode::Opaque
        } else if value == EnvironmentBlendMode::ADDITIVE {
            BlendMode::Additive
        } else if value == EnvironmentBlendMode::ALPHA_BLEND {
            BlendMode::AlphaBlend
        } else {
            unreachable!()
        }
    }
}

impl From<BlendMode> for openxr::EnvironmentBlendMode {
    fn from(value: BlendMode) -> Self {
        use openxr::EnvironmentBlendMode;
        match value {
            BlendMode::Opaque => EnvironmentBlendMode::OPAQUE,
            BlendMode::Additive => EnvironmentBlendMode::ADDITIVE,
            BlendMode::AlphaBlend => EnvironmentBlendMode::ALPHA_BLEND,
        }
    }
}
