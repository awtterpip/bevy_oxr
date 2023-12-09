mod vulkan;

use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{
    XrEnvironmentBlendMode, XrFormat, XrFrameState, XrFrameWaiter, XrInstance, XrResolution,
    XrSession, XrSessionRunning, XrSwapchain, XrViews,
};

use openxr as xr;

pub fn initialize_xr_graphics(
    window: Option<RawHandleWrapper>,
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
    vulkan::initialize_xr_graphics(window)
}

pub fn xr_entry() -> anyhow::Result<xr::Entry> {
    #[cfg(feature = "linked")]
    let entry = Ok(xr::Entry::linked());
    #[cfg(not(feature = "linked"))]
    let entry = unsafe { xr::Entry::load().map_err(|e| anyhow::anyhow!(e)) };
    entry
}
