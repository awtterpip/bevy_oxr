pub use openxr::{SwapchainCreateFlags, SwapchainUsageFlags, SystemId};

use super::init::{GraphicsExt, XrInitError};

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
    type Error = XrInitError;

    fn try_from(value: SwapchainCreateInfo) -> Result<Self, Self::Error> {
        Ok(openxr::SwapchainCreateInfo {
            create_flags: value.create_flags,
            usage_flags: value.usage_flags,
            format: G::from_wgpu_format(value.format)
                .ok_or(XrInitError::UnsupportedTextureFormat(value.format))?,
            sample_count: value.sample_count,
            width: value.width,
            height: value.height,
            face_count: value.face_count,
            array_size: value.array_size,
            mip_count: value.mip_count,
        })
    }
}
