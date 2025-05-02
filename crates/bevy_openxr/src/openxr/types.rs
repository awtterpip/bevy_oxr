use std::borrow::Cow;

use crate::error::OxrError;
use crate::graphics::GraphicsExt;

pub use crate::openxr::exts::OxrExtensions;

pub use openxr::{EnvironmentBlendMode, SwapchainCreateFlags, SwapchainUsageFlags};

pub type Result<T> = std::result::Result<T, OxrError>;

/// A container for all required graphics objects needed for a bevy app.
pub struct WgpuGraphics(
    pub wgpu::Device,
    pub wgpu::Queue,
    pub wgpu::AdapterInfo,
    pub wgpu::Adapter,
    pub wgpu::Instance,
);

/// A version number that can be stored inside of a u32
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Version(pub u8, pub u8, pub u16);

impl Version {
    /// Bevy's version number
    pub const BEVY: Self = Self(0, 15, 0);

    pub const fn to_u32(self) -> u32 {
        let major = (self.0 as u32) << 24;
        let minor = (self.1 as u32) << 16;
        self.2 as u32 | major | minor
    }
}

/// Info needed about an app for OpenXR
#[derive(Clone, Debug, PartialEq)]
pub struct AppInfo {
    pub name: Cow<'static, str>,
    pub version: Version,
}

impl AppInfo {
    /// The default app info for a generic bevy app
    pub const BEVY: Self = Self {
        name: Cow::Borrowed("Bevy"),
        version: Version::BEVY,
    };
}

impl Default for AppInfo {
    fn default() -> Self {
        Self::BEVY
    }
}

/// Info needed to create a swapchain.
/// This is an API agnostic version of [openxr::SwapchainCreateInfo] used for some of this library's functions
#[derive(Debug, Copy, Clone)]
pub struct SwapchainCreateInfo {
    pub create_flags: SwapchainCreateFlags,
    pub usage_flags: SwapchainUsageFlags,
    pub format: wgpu::TextureFormat,
    pub sample_count: u32,
    pub width: u32,
    pub height: u32,
    pub face_count: u32,
    pub array_size: u32,
    pub mip_count: u32,
}

impl<G: GraphicsExt> TryFrom<SwapchainCreateInfo> for openxr::SwapchainCreateInfo<G> {
    type Error = OxrError;

    fn try_from(value: SwapchainCreateInfo) -> Result<Self> {
        Ok(openxr::SwapchainCreateInfo {
            create_flags: value.create_flags,
            usage_flags: value.usage_flags,
            format: G::from_wgpu_format(value.format)
                .ok_or(OxrError::UnsupportedTextureFormat(value.format))?,
            sample_count: value.sample_count,
            width: value.width,
            height: value.height,
            face_count: value.face_count,
            array_size: value.array_size,
            mip_count: value.mip_count,
        })
    }
}
