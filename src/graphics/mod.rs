mod vulkan;

use bevy::render::renderer::{RenderDevice, RenderQueue, RenderAdapterInfo, RenderAdapter};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{XrInstance, XrSession, XrEnvironmentBlendMode, XrSessionRunning, XrFrameWaiter, XrSwapchain, XrViews};

pub fn initialize_xr_graphics(window: Option<RawHandleWrapper>) -> anyhow::Result<(RenderDevice, RenderQueue, RenderAdapterInfo, RenderAdapter, Instance, XrInstance, XrSession, XrEnvironmentBlendMode, XrSessionRunning, XrFrameWaiter, XrSwapchain, XrInput, XrViews)>{
    vulkan::initialize_xr_graphics(window)
}