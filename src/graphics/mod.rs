mod vulkan;

use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{
    XrEnvironmentBlendMode, XrFrameState, XrFrameWaiter, XrInstance, XrSession, XrSessionRunning,
    XrSwapchain, XrViews, XrResolution, XrFormat,
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

pub fn xr_entry() -> xr::Entry {
    #[cfg(feature = "linked")]
    let entry = xr::Entry::linked();
    #[cfg(not(feature = "linked"))]
    let entry = unsafe { xr::Entry::load().unwrap() };
    entry
}