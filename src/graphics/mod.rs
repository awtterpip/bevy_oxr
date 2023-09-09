mod vulkan;

use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{
    XrEnvironmentBlendMode, XrFrameState, XrFrameWaiter, XrInstance, XrSession, XrSessionRunning,
    XrSwapchain, XrViews,
};

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
    XrSessionRunning,
    XrFrameWaiter,
    XrSwapchain,
    XrInput,
    XrViews,
    XrFrameState,
)> {
    vulkan::initialize_xr_graphics(window)
}
