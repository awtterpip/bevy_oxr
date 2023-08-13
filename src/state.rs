use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use bevy::prelude::*;
use openxr as xr;
use xr::Result as XrResult;

#[derive(Resource)]
pub struct XrState {
    instance: xr::Instance,
    session: xr::Session<xr::AnyGraphics>,
    session_running: AtomicBool,
    event_buffer: xr::EventDataBuffer,
    views: Vec<xr::ViewConfigurationView>,
    graphics: XrGraphics,
}

enum XrGraphics {
    Vulkan(Mutex<XrGraphicsInner<xr::Vulkan>>)
}

impl XrGraphics {
    fn begin(&self) -> XrResult<xr::FrameState> {
        match self {
            XrGraphics::Vulkan(inner) => inner.lock().unwrap().begin(),
        }
    }
}

struct XrGraphicsInner<G: xr::Graphics> {
    wait: xr::FrameWaiter,
    stream: xr::FrameStream<G>,
    swapchain: xr::Swapchain<G>,
    blend_mode: xr::EnvironmentBlendMode,
    resolution: Extent2D,
    buffers: Vec<wgpu::Texture>,
}

impl<G: xr::Graphics> XrGraphicsInner<G> {
    fn begin(&mut self) -> XrResult<xr::FrameState> {
        let frame_state = self.wait.wait()?;
        self.stream.begin()?;
        Ok(frame_state)
    }

    fn get_render_view(&mut self, layer: u32) -> wgpu::TextureView {
        let image_index = self.swapchain.acquire_image().unwrap();
        self.swapchain.wait_image(xr::Duration::INFINITE).unwrap();

        let texture = &self.buffers[image_index as usize];

        texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            array_layer_count: Some(1),
            base_array_layer: layer,
            ..Default::default()
        })
    }
}

struct Extent2D {
    width: u32,
    height: u32,
}

unsafe impl Sync for XrState {}
unsafe impl Send for XrState {}