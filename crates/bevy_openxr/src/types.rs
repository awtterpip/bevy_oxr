use std::borrow::Cow;

pub use crate::error::XrError;
pub use crate::extensions::XrExtensions;
use crate::graphics::GraphicsExt;

pub use openxr::{
    ApiLayerProperties, EnvironmentBlendMode, SwapchainCreateFlags, SwapchainUsageFlags,
};

pub type Result<T> = std::result::Result<T, XrError>;

pub struct WgpuGraphics(
    pub wgpu::Device,
    pub wgpu::Queue,
    pub wgpu::AdapterInfo,
    pub wgpu::Adapter,
    pub wgpu::Instance,
);

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

#[derive(Clone, Debug, PartialEq)]
pub struct AppInfo {
    pub name: Cow<'static, str>,
    pub version: Version,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            name: "Bevy".into(),
            version: Version::BEVY,
        }
    }
}

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
    type Error = XrError;

    fn try_from(value: SwapchainCreateInfo) -> Result<Self> {
        Ok(openxr::SwapchainCreateInfo {
            create_flags: value.create_flags,
            usage_flags: value.usage_flags,
            format: G::from_wgpu_format(value.format)
                .ok_or(XrError::UnsupportedTextureFormat(value.format))?,
            sample_count: value.sample_count,
            width: value.width,
            height: value.height,
            face_count: value.face_count,
            array_size: value.array_size,
            mip_count: value.mip_count,
        })
    }
}
