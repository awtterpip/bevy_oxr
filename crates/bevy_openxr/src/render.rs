use bevy::{
    prelude::*,
    render::{
        camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews, RenderTarget},
        renderer::render_system,
        Render, RenderApp, RenderSet,
    },
};
use bevy_xr::camera::{XrCameraBundle, XrProjection, XrView};
use openxr::CompositionLayerFlags;

use crate::resources::*;
use crate::{init::begin_xr_session, layer_builder::*};

use crate::init::session_running;

pub struct XrRenderPlugin;

impl Plugin for XrRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            (
                init_views.run_if(resource_added::<XrGraphicsInfo>),
                wait_frame.run_if(session_running),
            )
                .after(begin_xr_session),
        );
        // .add_systems(Startup, init_views);
        app.sub_app_mut(RenderApp).add_systems(
            Render,
            (
                (begin_frame, insert_texture_views)
                    .chain()
                    .in_set(RenderSet::PrepareAssets)
                    .before(render_system),
                end_frame.in_set(RenderSet::Cleanup),
            )
                .run_if(session_running),
        );
    }
}

pub const XR_TEXTURE_INDEX: u32 = 3383858418;

// TODO: have cameras initialized externally and then recieved by this function.
/// This is needed to properly initialize the texture views so that bevy will set them to the correct resolution despite them being updated in the render world.
pub fn init_views(
    graphics_info: Res<XrGraphicsInfo>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    swapchain_images: Res<XrSwapchainImages>,
    mut commands: Commands,
) {
    let temp_tex = swapchain_images.first().unwrap();
    // this for loop is to easily add support for quad or mono views in the future.
    let mut views = vec![];
    for index in 0..2 {
        info!("{}", graphics_info.resolution);
        let view_handle =
            add_texture_view(&mut manual_texture_views, temp_tex, &graphics_info, index);

        let entity = commands
            .spawn(XrCameraBundle {
                camera: Camera {
                    target: RenderTarget::TextureView(view_handle),
                    ..Default::default()
                },
                ..Default::default()
            })
            .id();
        views.push(entity);
    }
    commands.insert_resource(XrViews(views));
}

pub fn wait_frame(mut frame_waiter: ResMut<XrFrameWaiter>, mut commands: Commands) {
    let state = frame_waiter.wait().expect("Failed to wait frame");
    // Here we insert the predicted display time for when this frame will be displayed.
    // TODO: don't add predicted_display_period if pipelined rendering plugin not enabled
    commands.insert_resource(XrTime(openxr::Time::from_nanos(
        state.predicted_display_time.as_nanos() + state.predicted_display_period.as_nanos(),
    )));
}

pub fn update_views(
    mut views: Query<(&mut Transform, &mut XrProjection), With<XrView>>,
    view_entities: Res<XrViews>,
    session: Res<XrSession>,
    stage: Res<XrStage>,
    time: Res<XrTime>,
) {
    let (_flags, xr_views) = session
        .locate_views(
            openxr::ViewConfigurationType::PRIMARY_STEREO,
            **time,
            &stage,
        )
        .expect("Failed to locate views");

    for (i, view) in xr_views.iter().enumerate() {
        if let Some((mut transform, mut projection)) = view_entities
            .0
            .get(i)
            .and_then(|entity| views.get_mut(*entity).ok())
        {
            let projection_matrix = calculate_projection(projection.near, view.fov);
            projection.projection_matrix = projection_matrix;

            let openxr::Quaternionf { x, y, z, w } = view.pose.orientation;
            let rotation = Quat::from_xyzw(x, y, z, w);
            transform.rotation = rotation;
            let openxr::Vector3f { x, y, z } = view.pose.position;
            let translation = Vec3::new(x, y, z);
            transform.translation = translation;
        }
    }
}

fn calculate_projection(near_z: f32, fov: openxr::Fovf) -> Mat4 {
    //  symmetric perspective for debugging
    // let x_fov = (self.fov.angle_left.abs() + self.fov.angle_right.abs());
    // let y_fov = (self.fov.angle_up.abs() + self.fov.angle_down.abs());
    // return Mat4::perspective_infinite_reverse_rh(y_fov, x_fov / y_fov, self.near);

    let is_vulkan_api = false; // FIXME wgpu probably abstracts this
    let far_z = -1.; //   use infinite proj
                     // let far_z = self.far;

    let tan_angle_left = fov.angle_left.tan();
    let tan_angle_right = fov.angle_right.tan();

    let tan_angle_down = fov.angle_down.tan();
    let tan_angle_up = fov.angle_up.tan();

    let tan_angle_width = tan_angle_right - tan_angle_left;

    // Set to tanAngleDown - tanAngleUp for a clip space with positive Y
    // down (Vulkan). Set to tanAngleUp - tanAngleDown for a clip space with
    // positive Y up (OpenGL / D3D / Metal).
    // const float tanAngleHeight =
    //     graphicsApi == GRAPHICS_VULKAN ? (tanAngleDown - tanAngleUp) : (tanAngleUp - tanAngleDown);
    let tan_angle_height = if is_vulkan_api {
        tan_angle_down - tan_angle_up
    } else {
        tan_angle_up - tan_angle_down
    };

    // Set to nearZ for a [-1,1] Z clip space (OpenGL / OpenGL ES).
    // Set to zero for a [0,1] Z clip space (Vulkan / D3D / Metal).
    // const float offsetZ =
    //     (graphicsApi == GRAPHICS_OPENGL || graphicsApi == GRAPHICS_OPENGL_ES) ? nearZ : 0;
    // FIXME handle enum of graphics apis
    let offset_z = 0.;

    let mut cols: [f32; 16] = [0.0; 16];

    if far_z <= near_z {
        // place the far plane at infinity
        cols[0] = 2. / tan_angle_width;
        cols[4] = 0.;
        cols[8] = (tan_angle_right + tan_angle_left) / tan_angle_width;
        cols[12] = 0.;

        cols[1] = 0.;
        cols[5] = 2. / tan_angle_height;
        cols[9] = (tan_angle_up + tan_angle_down) / tan_angle_height;
        cols[13] = 0.;

        cols[2] = 0.;
        cols[6] = 0.;
        cols[10] = -1.;
        cols[14] = -(near_z + offset_z);

        cols[3] = 0.;
        cols[7] = 0.;
        cols[11] = -1.;
        cols[15] = 0.;

        //  bevy uses the _reverse_ infinite projection
        //  https://dev.theomader.com/depth-precision/
        let z_reversal = Mat4::from_cols_array_2d(&[
            [1f32, 0., 0., 0.],
            [0., 1., 0., 0.],
            [0., 0., -1., 0.],
            [0., 0., 1., 1.],
        ]);

        return z_reversal * Mat4::from_cols_array(&cols);
    } else {
        // normal projection
        cols[0] = 2. / tan_angle_width;
        cols[4] = 0.;
        cols[8] = (tan_angle_right + tan_angle_left) / tan_angle_width;
        cols[12] = 0.;

        cols[1] = 0.;
        cols[5] = 2. / tan_angle_height;
        cols[9] = (tan_angle_up + tan_angle_down) / tan_angle_height;
        cols[13] = 0.;

        cols[2] = 0.;
        cols[6] = 0.;
        cols[10] = -(far_z + offset_z) / (far_z - near_z);
        cols[14] = -(far_z * (near_z + offset_z)) / (far_z - near_z);

        cols[3] = 0.;
        cols[7] = 0.;
        cols[11] = -1.;
        cols[15] = 0.;
    }

    Mat4::from_cols_array(&cols)
}

pub fn begin_frame(mut frame_stream: ResMut<XrFrameStream>) {
    frame_stream.begin().expect("Failed to begin frame");
}

pub fn insert_texture_views(
    swapchain_images: Res<XrSwapchainImages>,
    mut swapchain: ResMut<XrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    graphics_info: Res<XrGraphicsInfo>,
) {
    let index = swapchain.acquire_image().expect("Failed to acquire image");
    swapchain
        .wait_image(openxr::Duration::INFINITE)
        .expect("Failed to wait image");
    let image = &swapchain_images[index as usize];

    for i in 0..2 {
        add_texture_view(&mut manual_texture_views, image, &graphics_info, i);
    }
}

pub fn add_texture_view(
    manual_texture_views: &mut ManualTextureViews,
    texture: &wgpu::Texture,
    info: &XrGraphicsInfo,
    index: u32,
) -> ManualTextureViewHandle {
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::D2),
        array_layer_count: Some(1),
        base_array_layer: index,
        ..default()
    });
    let view = ManualTextureView {
        texture_view: view.into(),
        size: info.resolution,
        format: info.format,
    };
    let handle = ManualTextureViewHandle(XR_TEXTURE_INDEX + index);
    manual_texture_views.insert(handle, view);
    handle
}

pub fn end_frame(
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
            width: graphics_info.resolution.x as _,
            height: graphics_info.resolution.y as _,
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
                        .pose(views[1].pose)
                        .fov(views[1].fov)
                        .sub_image(
                            SwapchainSubImage::new()
                                .swapchain(&swapchain)
                                .image_array_index(1)
                                .image_rect(rect),
                        ),
                ])],
        )
        .expect("Failed to end stream");
}
