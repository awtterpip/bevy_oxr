pub mod extensions;
mod vulkan;

use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::passthrough::{Passthrough, PassthroughLayer};
use crate::resources::{
    XrEnvironmentBlendMode, XrFormat, XrFrameState, XrFrameWaiter, XrInstance, XrPassthrough,
    XrPassthroughLayer, XrResolution, XrSession, XrSessionRunning, XrSwapchain, XrViews,
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
) -> anyhow::Result<(
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
    vulkan::initialize_xr_graphics(window, reqeusted_extensions, prefered_blend_mode, app_info)
}

pub fn xr_entry() -> anyhow::Result<xr::Entry> {
    #[cfg(windows)]
    let entry = Ok(xr::Entry::linked());
    #[cfg(not(windows))]
    let entry = unsafe { xr::Entry::load().map_err(|e| anyhow::anyhow!(e)) };
    entry
}
