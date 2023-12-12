#[cfg(all(feature = "d3d12", windows))]
mod d3d12;
#[cfg(feature = "vulkan")]
mod vulkan;

use anyhow::bail;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue};
use bevy::window::RawHandleWrapper;
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{
    XrEnvironmentBlendMode, XrFormat, XrFrameState, XrFrameWaiter, XrInstance, XrResolution,
    XrSession, XrSessionRunning, XrSwapchain, XrViews,
};
use crate::Backend;

use openxr as xr;

pub fn initialize_xr_graphics(
    backend_preference: &[Backend],
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
    if backend_preference.is_empty() {
        bail!("Cannot initialize with no backend selected");
    }
    let xr_entry = xr_entry()?;
    
    #[cfg(target_os = "android")]
    xr_entry.initialize_android_loader()?;

    let available_extensions = xr_entry.enumerate_extensions()?;

    for backend in backend_preference {
        match backend {
            #[cfg(feature = "vulkan")]
            Backend::Vulkan => {
                if !available_extensions.khr_vulkan_enable2 {
                    continue;
                }
                return vulkan::initialize_xr_graphics(window, xr_entry, available_extensions)
            }
            #[cfg(all(feature = "d3d12", windows))]
            Backend::D3D12 => {
                if !available_extensions.khr_d3d12_enable {
                    continue;
                }
                return d3d12::initialize_xr_graphics(window, xr_entry, available_extensions)
            }
        }
    }
    bail!("No selected backend was supported by the runtime. Selected: ");
}

pub fn xr_entry() -> anyhow::Result<xr::Entry> {
    #[cfg(feature = "linked")]
    let entry = Ok(xr::Entry::linked());
    #[cfg(not(feature = "linked"))]
    let entry = unsafe { xr::Entry::load().map_err(|e| anyhow::anyhow!(e)) };
    entry
}
