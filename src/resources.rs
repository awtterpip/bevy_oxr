use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::resource_macros::*;
use bevy::prelude::*;
use openxr as xr;

xr_resource_wrapper!(XrInstance, xr::Instance);
xr_resource_wrapper!(XrSession, xr::Session<xr::AnyGraphics>);
xr_resource_wrapper!(XrEnvironmentBlendMode, xr::EnvironmentBlendMode);
xr_resource_wrapper!(XrViewConfigurationViews, Vec<xr::ViewConfigurationView>);
xr_arc_resource_wrapper!(XrSessionRunning, AtomicBool);
xr_arc_resource_wrapper!(XrFrameWaiter, Mutex<xr::FrameWaiter>);
xr_arc_resource_wrapper!(XrSwapchain, Mutex<Swapchain>);
xr_arc_resource_wrapper!(XrFrameState, Mutex<Option<xr::FrameState>>);
xr_arc_resource_wrapper!(XrViews, Mutex<Vec<xr::View>>);

pub enum Swapchain {
    Vulkan(SwapchainInner<xr::Vulkan>),
}

impl Swapchain {
    pub(crate) fn begin(&mut self) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swap) => swap.begin(),
        }
    }

    pub(crate) fn update_render_views(&mut self) {
        match self {
            Swapchain::Vulkan(swap) => swap.update_render_views(),
        }
    }

    pub(crate) fn get_render_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        match self {
            Swapchain::Vulkan(swap) => swap.get_render_views(),
        }
    }

    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        match self {
            Swapchain::Vulkan(swap) => swap.format
        }
    }

    pub(crate) fn resolution(&self) -> UVec2 {
        match self {
            Swapchain::Vulkan(swap) => swap.resolution,
        }
    }

    pub(crate) fn post_queue_submit(
        &mut self,
        xr_frame_state: xr::FrameState,
        views: &[openxr::View],
        stage: &xr::Space,
        environment_blend_mode: xr::EnvironmentBlendMode,
    ) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swap) => swap.post_queue_submit(xr_frame_state, views, stage, environment_blend_mode),
        }
    }
}

pub struct SwapchainInner<G: xr::Graphics> {
    pub(crate) stream: xr::FrameStream<G>,
    pub(crate) handle: xr::Swapchain<G>,
    pub(crate) resolution: UVec2,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) buffers: Vec<wgpu::Texture>,
    pub(crate) image_index: usize,
}

impl<G: xr::Graphics> SwapchainInner<G> {
    fn begin(&mut self) -> xr::Result<()> {
        self.stream.begin()
    }

    fn get_render_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        let texture = &self.buffers[self.image_index];

        (
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1),
                ..Default::default()
            }),
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1),
                base_array_layer: 1,
                ..Default::default()
            }),
        )
    }

    fn update_render_views(&mut self) {
        let image_index = self.handle.acquire_image().unwrap();
        self.handle.wait_image(xr::Duration::INFINITE).unwrap();

        self.image_index = image_index as _;
    }

    fn post_queue_submit(
        &mut self,
        xr_frame_state: xr::FrameState,
        views: &[openxr::View],
        stage: &xr::Space,
        environment_blend_mode: xr::EnvironmentBlendMode,
    ) -> xr::Result<()> {
        self.handle.release_image().unwrap();
            let rect = xr::Rect2Di {
                offset: xr::Offset2Di { x: 0, y: 0 },
                extent: xr::Extent2Di {
                    width: self.resolution.x as _,
                    height: self.resolution.y as _,
                },
            };
            self.stream.end(
                xr_frame_state.predicted_display_time,
                environment_blend_mode,
                &[&xr::CompositionLayerProjection::new().space(stage).views(&[
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[0].pose)
                        .fov(views[0].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&self.handle)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[1].pose)
                        .fov(views[1].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&self.handle)
                                .image_array_index(1)
                                .image_rect(rect),
                        ),
                ])],
            )
    }
}
