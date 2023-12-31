use std::ptr;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::passthrough::XrPassthroughLayer;
use crate::resource_macros::*;
use bevy::prelude::*;
use openxr as xr;
use xr::sys::CompositionLayerPassthroughFB;
use xr::{CompositionLayerBase, CompositionLayerFlags, Space};

xr_resource_wrapper!(XrInstance, xr::Instance);
xr_resource_wrapper!(XrSession, xr::Session<xr::AnyGraphics>);
xr_resource_wrapper!(XrEnvironmentBlendMode, xr::EnvironmentBlendMode);
xr_resource_wrapper!(XrResolution, UVec2);
xr_resource_wrapper!(XrFormat, wgpu::TextureFormat);
xr_arc_resource_wrapper!(XrSessionRunning, AtomicBool);
xr_arc_resource_wrapper!(XrFrameWaiter, Mutex<xr::FrameWaiter>);
xr_arc_resource_wrapper!(XrSwapchain, Swapchain);
xr_arc_resource_wrapper!(XrFrameState, Mutex<xr::FrameState>);
xr_arc_resource_wrapper!(XrViews, Mutex<Vec<xr::View>>);

pub enum Swapchain {
    Vulkan(SwapchainInner<xr::Vulkan>),
}

impl Swapchain {
    pub(crate) fn begin(&self) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.begin(),
        }
    }

    pub(crate) fn get_render_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.get_render_views(),
        }
    }

    pub(crate) fn acquire_image(&self) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.acquire_image(),
        }
    }

    pub(crate) fn wait_image(&self) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.wait_image(),
        }
    }

    pub(crate) fn release_image(&self) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.release_image(),
        }
    }

    pub(crate) fn end(
        &self,
        predicted_display_time: xr::Time,
        views: &[openxr::View],
        stage: &xr::Space,
        resolution: UVec2,
        environment_blend_mode: xr::EnvironmentBlendMode,
        passthrough_layer: Option<&XrPassthroughLayer>,
    ) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.end(
                predicted_display_time,
                views,
                stage,
                resolution,
                environment_blend_mode,
                passthrough_layer,
            ),
        }
    }
}

pub struct SwapchainInner<G: xr::Graphics> {
    pub(crate) stream: Mutex<xr::FrameStream<G>>,
    pub(crate) handle: Mutex<xr::Swapchain<G>>,
    pub(crate) buffers: Vec<wgpu::Texture>,
    pub(crate) image_index: Mutex<usize>,
}

impl<G: xr::Graphics> SwapchainInner<G> {
    fn begin(&self) -> xr::Result<()> {
        self.stream.lock().unwrap().begin()
    }

    fn get_render_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        let texture = &self.buffers[*self.image_index.lock().unwrap()];

        (
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1),
                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                ..Default::default()
            }),
            texture.create_view(&wgpu::TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                array_layer_count: Some(1),
                base_array_layer: 1,
                format: Some(wgpu::TextureFormat::Rgba8UnormSrgb),
                ..Default::default()
            }),
        )
    }

    fn acquire_image(&self) -> xr::Result<()> {
        let image_index = self.handle.lock().unwrap().acquire_image()?;
        *self.image_index.lock().unwrap() = image_index as _;
        Ok(())
    }

    fn wait_image(&self) -> xr::Result<()> {
        self.handle
            .lock()
            .unwrap()
            .wait_image(xr::Duration::INFINITE)
    }

    fn release_image(&self) -> xr::Result<()> {
        self.handle.lock().unwrap().release_image()
    }

    fn end(
        &self,
        predicted_display_time: xr::Time,
        views: &[openxr::View],
        stage: &xr::Space,
        resolution: UVec2,
        environment_blend_mode: xr::EnvironmentBlendMode,
        passthrough_layer: Option<&XrPassthroughLayer>,
    ) -> xr::Result<()> {
        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di {
                width: resolution.x as _,
                height: resolution.y as _,
            },
        };
        let swapchain = self.handle.lock().unwrap();
        if views.len() == 0 {
            warn!("views are len of 0");
            return Ok(());
        }
        match passthrough_layer {
            Some(pass) => {
                // info!("Rendering with pass through");
                let passthrough_layer = xr::sys::CompositionLayerPassthroughFB {
                    ty: CompositionLayerPassthroughFB::TYPE,
                    next: ptr::null(),
                    flags: CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA,
                    space: xr::sys::Space::NULL,
                    layer_handle: pass.0,
                };
                self.stream.lock().unwrap().end(
                    predicted_display_time,
                    environment_blend_mode,
                    &[
                        &xr::CompositionLayerProjection::new()
                            .layer_flags(CompositionLayerFlags::UNPREMULTIPLIED_ALPHA)
                            .space(stage)
                            .views(&[
                                xr::CompositionLayerProjectionView::new()
                                    .pose(views[0].pose)
                                    .fov(views[0].fov)
                                    .sub_image(
                                        xr::SwapchainSubImage::new()
                                            .swapchain(&swapchain)
                                            .image_array_index(0)
                                            .image_rect(rect),
                                    ),
                                xr::CompositionLayerProjectionView::new()
                                    .pose(views[1].pose)
                                    .fov(views[1].fov)
                                    .sub_image(
                                        xr::SwapchainSubImage::new()
                                            .swapchain(&swapchain)
                                            .image_array_index(1)
                                            .image_rect(rect),
                                    ),
                            ]),
                        unsafe {
                            &*(&passthrough_layer as *const _ as *const CompositionLayerBase<G>)
                        },
                    ],
                )
            }

            None => self.stream.lock().unwrap().end(
                predicted_display_time,
                environment_blend_mode,
                &[&xr::CompositionLayerProjection::new().space(stage).views(&[
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[0].pose)
                        .fov(views[0].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                    xr::CompositionLayerProjectionView::new()
                        .pose(views[1].pose)
                        .fov(views[1].fov)
                        .sub_image(
                            xr::SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(1)
                                .image_rect(rect),
                        ),
                ])],
            ),
        }
    }
}
