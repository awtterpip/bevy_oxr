use std::ffi::c_void;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use crate::input::XrInput;
// use crate::passthrough::XrPassthroughLayer;
use crate::xr_init::XrStatus;
use crate::{resource_macros::*, xr_resource_wrapper_no_extract};
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use openxr as xr;

xr_resource_wrapper!(XrInstance, xr::Instance);
xr_resource_wrapper!(XrSession, xr::Session<xr::AnyGraphics>);
xr_resource_wrapper!(XrEnvironmentBlendMode, xr::EnvironmentBlendMode);
xr_resource_wrapper!(XrResolution, UVec2);
xr_resource_wrapper!(XrFormat, wgpu::TextureFormat);
xr_resource_wrapper_no_extract!(XrFrameState, xr::FrameState);
xr_resource_wrapper!(XrViews, Vec<xr::View>);
xr_arc_resource_wrapper!(XrSessionRunning, AtomicBool);
xr_arc_resource_wrapper!(XrSwapchain, Swapchain);
xr_no_clone_resource_wrapper!(XrFrameWaiter, xr::FrameWaiter);

impl ExtractResource for XrFrameState {
    type Source = Self;

    fn extract_resource(source: &Self::Source) -> Self {
        let mut state = *source;
        state.predicted_display_time = xr::Time::from_nanos(
            state.predicted_display_time.as_nanos() + state.predicted_display_period.as_nanos(),
        );
        state
    }
}

pub(crate) struct VulkanOXrSessionSetupInfo {
    pub(crate) device_ptr: *const c_void,
    pub(crate) physical_device_ptr: *const c_void,
    pub(crate) vk_instance_ptr: *const c_void,
    pub(crate) queue_family_index: u32,
    pub(crate) xr_system_id: xr::SystemId,
}

pub(crate) enum OXrSessionSetupInfo {
    Vulkan(VulkanOXrSessionSetupInfo),
}

pub struct XrResourcePlugin;

impl Plugin for XrResourcePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<XrResolution>::default());
        app.add_plugins(ExtractResourcePlugin::<XrFormat>::default());
        app.add_plugins(ExtractResourcePlugin::<XrSwapchain>::default());
        app.add_plugins(ExtractResourcePlugin::<XrFrameState>::default());
        app.add_plugins(ExtractResourcePlugin::<XrViews>::default());
        app.add_plugins(ExtractResourcePlugin::<XrInput>::default());
        app.add_plugins(ExtractResourcePlugin::<XrEnvironmentBlendMode>::default());
        app.add_plugins(ExtractResourcePlugin::<XrSessionRunning>::default());
        app.add_plugins(ExtractResourcePlugin::<XrSession>::default());
    }
}

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
        // passthrough_layer: Option<&XrPassthroughLayer>,
    ) -> xr::Result<()> {
        match self {
            Swapchain::Vulkan(swapchain) => swapchain.end(
                predicted_display_time,
                views,
                stage,
                resolution,
                environment_blend_mode,
                // passthrough_layer,
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
        // passthrough_layer: Option<&XrPassthroughLayer>,
    ) -> xr::Result<()> {
        let rect = xr::Rect2Di {
            offset: xr::Offset2Di { x: 0, y: 0 },
            extent: xr::Extent2Di {
                width: resolution.x as _,
                height: resolution.y as _,
            },
        };
        let swapchain = self.handle.lock().unwrap();
        if views.is_empty() {
            warn!("views are len of 0");
            return Ok(());
        }
        // match passthrough_layer {
        //     Some(pass) => {
        //         // info!("Rendering with pass through");
        //         let passthrough_layer = xr::sys::CompositionLayerPassthroughFB {
        //             ty: CompositionLayerPassthroughFB::TYPE,
        //             next: ptr::null(),
        //             flags: CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA,
        //             space: xr::sys::Space::NULL,
        //             layer_handle: pass.0,
        //         };
        //         self.stream.lock().unwrap().end(
        //             predicted_display_time,
        //             environment_blend_mode,
        //             &[
        //                 &xr::CompositionLayerProjection::new()
        //                     .layer_flags(CompositionLayerFlags::UNPREMULTIPLIED_ALPHA)
        //                     .space(stage)
        //                     .views(&[
        //                         xr::CompositionLayerProjectionView::new()
        //                             .pose(views[0].pose)
        //                             .fov(views[0].fov)
        //                             .sub_image(
        //                                 xr::SwapchainSubImage::new()
        //                                     .swapchain(&swapchain)
        //                                     .image_array_index(0)
        //                                     .image_rect(rect),
        //                             ),
        //                         xr::CompositionLayerProjectionView::new()
        //                             .pose(views[1].pose)
        //                             .fov(views[1].fov)
        //                             .sub_image(
        //                                 xr::SwapchainSubImage::new()
        //                                     .swapchain(&swapchain)
        //                                     .image_array_index(1)
        //                                     .image_rect(rect),
        //                             ),
        //                     ]),
        //                 unsafe {
        //                     &*(&passthrough_layer as *const _ as *const CompositionLayerBase<G>)
        //                 },
        //             ],
        //         )
        //     }

        // None =>
        let r = self.stream.lock().unwrap().end(
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
        );
        r
        // }
    }
}
