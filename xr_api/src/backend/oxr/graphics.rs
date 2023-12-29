mod vulkan;

use super::OXrSession;
use crate::error::{Result, XrError};
use crate::types::ExtensionSet;

pub fn init_oxr_graphics(
    instance: openxr::Instance,
    extensions: ExtensionSet,
    format: wgpu::TextureFormat,
) -> Result<OXrSession> {
    if extensions.vulkan {
        return vulkan::init_oxr_graphics(instance, format);
    }

    Err(XrError::Placeholder)
}
