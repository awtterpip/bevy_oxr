pub mod extensions;
mod vulkan;

use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{
    XrEnvironmentBlendMode, XrFormat, XrFrameState, XrInstance, XrResolution, XrSession,
    XrSessionRunning, XrSwapchain, XrViews, XrFrameWaiter,
};

use openxr as xr;

use self::extensions::XrExtensions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrPreferdBlendMode {
    Opaque,
    Additive,
    AlphaBlend,
}
impl Default for XrPreferdBlendMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Clone, Debug)]
pub struct XrAppInfo {
    pub name: String,
}
impl Default for XrAppInfo {
    fn default() -> Self {
        Self {
            name: "Ambient".into(),
        }
    }
}

pub fn initialize_xr_graphics(
    window: Option<RawHandleWrapper>,
    reqeusted_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
) -> eyre::Result<(
    RenderDevice,
    RenderQueue,
    RenderAdapterInfo,
    RenderAdapter,
    Instance,
    XrInstance,
    XrSession,
    XrEnvironmentBlendMode,
    XrResolution,
    XrFormat,
    XrSessionRunning,
    XrFrameWaiter,
    XrSwapchain,
    XrInput,
    XrViews,
    XrFrameState,
)> {
    // vulkan::initialize_xr_graphics(window, reqeusted_extensions, prefered_blend_mode, app_info)
    todo!()
}

pub fn xr_entry() -> eyre::Result<xr::Entry> {
    #[cfg(windows)]
    let entry = Ok(xr::Entry::linked());
    #[cfg(not(windows))]
    let entry = unsafe { xr::Entry::load().map_err(|e| eyre::eyre!(e)) };
    entry
}
