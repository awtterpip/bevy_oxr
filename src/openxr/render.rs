use bevy::{
    prelude::*,
    render::{
        camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews},
        Render, RenderApp, RenderSet,
    },
};
use openxr::CompositionLayerFlags;

use crate::openxr::resources::*;
use crate::openxr::types::*;
use crate::openxr::XrTime;

use super::{poll_events, session_running};

pub struct XrRenderPlugin;

impl Plugin for XrRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            wait_frame.after(poll_events).run_if(session_running()),
        )
        .add_systems(Startup, init_texture_views);
        app.sub_app_mut(RenderApp).add_systems(
            Render,
            (
                (begin_frame, insert_texture_views)
                    .chain()
                    .in_set(RenderSet::PrepareAssets),
                end_frame.in_set(RenderSet::Cleanup),
            )
                .run_if(session_running()),
        );
    }
}

pub const LEFT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(1208214591);
pub const RIGHT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(3383858418);

fn init_texture_views(
    graphics_info: Res<XrGraphicsInfo>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    swapchain_images: Res<SwapchainImages>,
) {
    let temp_tex = swapchain_images.first().unwrap();
    let left = temp_tex.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        array_layer_count: Some(1),
        ..Default::default()
    });
    let right = temp_tex.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        array_layer_count: Some(1),
        base_array_layer: 1,
        ..Default::default()
    });
    let resolution = graphics_info.swapchain_resolution;
    let format = graphics_info.swapchain_format;
    let left = ManualTextureView {
        texture_view: left.into(),
        size: resolution,
        format: format,
    };
    let right = ManualTextureView {
        texture_view: right.into(),
        size: resolution,
        format: format,
    };
    manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
    manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
}

fn wait_frame(mut frame_waiter: ResMut<XrFrameWaiter>, mut commands: Commands) {
    let state = frame_waiter.wait().expect("Failed to wait frame");
    commands.insert_resource(XrTime(openxr::Time::from_nanos(
        state.predicted_display_time.as_nanos() + state.predicted_display_period.as_nanos(),
    )));
}

pub fn begin_frame(mut frame_stream: ResMut<XrFrameStream>) {
    frame_stream.begin().expect("Failed to begin frame");
}

fn insert_texture_views(
    swapchain_images: Res<SwapchainImages>,
    mut swapchain: ResMut<XrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    graphics_info: Res<XrGraphicsInfo>,
) {
    let index = swapchain.acquire_image().expect("Failed to acquire image");
    swapchain
        .wait_image(openxr::Duration::INFINITE)
        .expect("Failed to wait image");
    let image = &swapchain_images[index as usize];
    let left = image.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        array_layer_count: Some(1),
        ..Default::default()
    });
    let right = image.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        array_layer_count: Some(1),
        base_array_layer: 1,
        ..Default::default()
    });
    let resolution = graphics_info.swapchain_resolution;
    let format = graphics_info.swapchain_format;
    let left = ManualTextureView {
        texture_view: left.into(),
        size: resolution,
        format: format,
    };
    let right = ManualTextureView {
        texture_view: right.into(),
        size: resolution,
        format: format,
    };
    manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
    manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
}

fn end_frame(
    mut frame_stream: ResMut<XrFrameStream>,
    session: Res<XrSession>,
    mut swapchain: ResMut<XrSwapchain>,
    stage: Res<XrStage>,
    display_time: Res<XrTime>,
    graphics_info: Res<XrGraphicsInfo>,
) {
    swapchain.release_image().unwrap();
    let (_flags, views) = session
        .locate_views(
            openxr::ViewConfigurationType::PRIMARY_STEREO,
            **display_time,
            &stage,
        )
        .expect("Failed to locate views");

    let rect = openxr::Rect2Di {
        offset: openxr::Offset2Di { x: 0, y: 0 },
        extent: openxr::Extent2Di {
            width: graphics_info.swapchain_resolution.x as _,
            height: graphics_info.swapchain_resolution.y as _,
        },
    };
    frame_stream
        .end(
            **display_time,
            graphics_info.blend_mode,
            &[&CompositionLayerProjection::new()
                .layer_flags(CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA)
                .space(&stage)
                .views(&[
                    CompositionLayerProjectionView::new()
                        .pose(views[0].pose)
                        .fov(views[0].fov)
                        .sub_image(
                            SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                    CompositionLayerProjectionView::new()
                        .pose(views[0].pose)
                        .fov(views[0].fov)
                        .sub_image(
                            SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(0)
                                .image_rect(rect),
                        ),
                ])],
        )
        .expect("Failed to end stream");
}
