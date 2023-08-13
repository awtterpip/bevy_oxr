use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderQueue, RenderDevice};
use bevy::render::settings::WgpuSettings;
use openxr::Entry;
use wgpu::Instance;

/// Initializes the renderer by retrieving and preparing the GPU instance, device and queue
/// for the specified backend.
pub fn initialize_renderer(
    options: &WgpuSettings,
    entry: &Entry,
) -> (RenderDevice, RenderQueue, RenderAdapterInfo, RenderAdapter, Instance) {
    todo!()
}