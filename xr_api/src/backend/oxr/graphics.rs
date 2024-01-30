mod vulkan;

use std::sync::Mutex;

use glam::UVec2;
use tracing::warn;

use super::{OXrInstance, OXrSession};
use crate::error::{Result, XrError};
use crate::types::ExtensionSet;

pub const VIEW_TYPE: openxr::ViewConfigurationType = openxr::ViewConfigurationType::PRIMARY_STEREO;

pub fn init_oxr_graphics(
    instance: OXrInstance,
    extensions: ExtensionSet,
    format: wgpu::TextureFormat,
) -> Result<OXrSession> {
    if extensions.vulkan {
        return vulkan::init_oxr_graphics(instance, format, String::new());
    }

    Err(XrError::Placeholder)
}

pub enum Swapchain {
    Vulkan(SwapchainInner<openxr::Vulkan>),
}

impl Swapchain {
    pub(crate) fn begin(&self) -> Result<()> {
        Ok(match self {
            Swapchain::Vulkan(swapchain) => swapchain.begin()?,
        })
    }

    pub(crate) fn get_render_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.get_render_views(),
        }
    }

    pub(crate) fn acquire_image(&self) -> Result<()> {
        Ok(match self {
            Swapchain::Vulkan(swapchain) => swapchain.acquire_image()?,
        })
    }

    pub(crate) fn wait_image(&self) -> Result<()> {
        Ok(match self {
            Swapchain::Vulkan(swapchain) => swapchain.wait_image()?,
        })
    }

    pub(crate) fn release_image(&self) -> Result<()> {
        Ok(match self {
            Swapchain::Vulkan(swapchain) => swapchain.release_image()?,
        })
    }

    pub(crate) fn end(
        &self,
        predicted_display_time: openxr::Time,
        views: &[openxr::View],
        stage: &openxr::Space,
        resolution: UVec2,
        environment_blend_mode: openxr::EnvironmentBlendMode,
        should_render: bool,
    ) -> Result<()> {
        Ok(match self {
            Swapchain::Vulkan(swapchain) => swapchain.end(
                predicted_display_time,
                views,
                stage,
                resolution,
                environment_blend_mode,
                should_render,
            )?,
        })
    }
}

pub struct SwapchainInner<G: openxr::Graphics> {
    stream: Mutex<openxr::FrameStream<G>>,
    swapchain: Mutex<openxr::Swapchain<G>>,
    buffers: Vec<wgpu::Texture>,
    image_index: Mutex<usize>,
}

impl<G: openxr::Graphics> SwapchainInner<G> {
    fn begin(&self) -> openxr::Result<()> {
        self.stream.lock().unwrap().begin()
    }

    fn get_render_views(&self) -> (wgpu::TextureView, wgpu::TextureView) {
        let texture = &self.buffers[*self.image_index.lock().unwrap()];

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

    fn acquire_image(&self) -> openxr::Result<()> {
        let image_index = self.swapchain.lock().unwrap().acquire_image()?;
        *self.image_index.lock().unwrap() = image_index as _;
        Ok(())
    }

    fn wait_image(&self) -> openxr::Result<()> {
        self.swapchain
            .lock()
            .unwrap()
            .wait_image(openxr::Duration::INFINITE)
    }

    fn release_image(&self) -> openxr::Result<()> {
        self.swapchain.lock().unwrap().release_image()
    }

    fn end(
        &self,
        predicted_display_time: openxr::Time,
        views: &[openxr::View],
        stage: &openxr::Space,
        resolution: UVec2,
        environment_blend_mode: openxr::EnvironmentBlendMode,
        should_render: bool,
    ) -> openxr::Result<()> {
        let rect = openxr::Rect2Di {
            offset: openxr::Offset2Di { x: 0, y: 0 },
            extent: openxr::Extent2Di {
                width: resolution.x as _,
                height: resolution.y as _,
            },
        };
        let swapchain = self.swapchain.lock().unwrap();
        if views.len() == 0 {
            warn!("views are len of 0");
            return Ok(());
        }
        let mut stream = self.stream.lock().unwrap();
        if true {
            stream.end(
                predicted_display_time,
                environment_blend_mode,
                &[&openxr::CompositionLayerProjection::new()
                    .space(stage)
                    .views(&[
                        openxr::CompositionLayerProjectionView::new()
                            .pose(views[0].pose)
                            .fov(views[0].fov)
                            .sub_image(
                                openxr::SwapchainSubImage::new()
                                    .swapchain(&swapchain)
                                    .image_array_index(0)
                                    .image_rect(rect),
                            ),
                        openxr::CompositionLayerProjectionView::new()
                            .pose(views[1].pose)
                            .fov(views[1].fov)
                            .sub_image(
                                openxr::SwapchainSubImage::new()
                                    .swapchain(&swapchain)
                                    .image_array_index(1)
                                    .image_rect(rect),
                            ),
                    ])],
            )
        } else {
            stream.end(predicted_display_time, environment_blend_mode, &[])
        }
    }
}
