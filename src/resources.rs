use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use crate::resource_macros::*;
use openxr as xr;

xr_resource_wrapper!(XrInstance, xr::Instance);
xr_resource_wrapper!(XrSession, xr::Session<xr::AnyGraphics>);
xr_resource_wrapper!(XrEnvironmentBlendMode, xr::EnvironmentBlendMode);
xr_arc_resource_wrapper!(XrSessionRunning, AtomicBool);
xr_arc_resource_wrapper!(XrFrameWaiter, Mutex<XrFrameWaiter>);
xr_arc_resource_wrapper!(XrSwapchain, Mutex<Swapchain>);

pub enum Swapchain {
    Vulkan(SwapchainInner<xr::Vulkan>)
}

pub struct SwapchainInner<G: xr::Graphics> {
    stream: xr::FrameStream<G>,
    handle: xr::Swapchain<G>,
    resolution: (u32, u32),
    buffers: Vec<wgpu::Texture>,
}