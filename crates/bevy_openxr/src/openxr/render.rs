use std::sync::mpsc::{sync_channel, Receiver, SyncSender};

use bevy::{
    ecs::{query::QuerySingleError, schedule::SystemConfigs},
    prelude::*,
    render::{
        camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews, RenderTarget},
        renderer::{render_system, RenderDevice},
        view::ExtractedView,
        Render, RenderApp, RenderSet,
    },
    utils::synccell::SyncCell,
};
use bevy_xr::camera::{XrCamera, XrCameraBundle, XrProjection};
use openxr::ViewStateFlags;

use crate::{
    error::OxrError,
    init::{
        session_started, OxrPreUpdateSet, OxrSessionResourceCreator, OxrSessionResourceCreators,
        OxrTrackingRoot,
    },
    layer_builder::ProjectionLayer,
};
use crate::{reference_space::OxrPrimaryReferenceSpace, resources::*, types::*};

fn update_view_systems() -> SystemConfigs {
    (
        locate_views,
        update_views.run_if(resource_exists::<OxrViews>),
    )
        .chain()
        .run_if(session_started)
}

pub struct OxrRenderPlugin;

impl Plugin for OxrRenderPlugin {
    fn build(&self, app: &mut App) {
        app.world
            .resource::<OxrSessionResourceCreators>()
            .init_resource_creator::<OxrSwapchainCreator>();
        app.add_systems(
            PreUpdate,
            wait_frame
                .run_if(session_started)
                .in_set(OxrPreUpdateSet::WaitFrame),
        )
        .add_systems(
            PreUpdate,
            (
                init_views.run_if(resource_added::<OxrGraphicsInfo>),
                update_view_systems(),
            )
                .chain()
                .after(OxrPreUpdateSet::UpdateNonCriticalComponents),
        );

        app.sub_app_mut(RenderApp)
            .add_systems(
                Render,
                (
                    (
                        begin_frame,
                        insert_texture_views,
                        locate_views.run_if(resource_exists::<OxrPrimaryReferenceSpace>),
                        update_views_render_world,
                    )
                        .chain()
                        .in_set(RenderSet::PrepareAssets),
                    wait_image.in_set(RenderSet::Render).before(render_system),
                    (release_image, end_frame)
                        .chain()
                        .in_set(RenderSet::Cleanup),
                )
                    .run_if(session_started),
            )
            .insert_resource(OxrViews(vec![]))
            .insert_resource(OxrRenderLayers(vec![Box::new(ProjectionLayer)]));
    }
}

#[derive(Default)]
struct OxrSwapchainCreator {
    swapchain: Option<OxrSwapchain>,
    images: Option<OxrSwapchainImages>,
    graphics_info: Option<OxrGraphicsInfo>,
}

impl OxrSessionResourceCreator for OxrSwapchainCreator {
    fn update(&mut self, world: &mut World) -> Result<()> {
        let instance = world.resource::<OxrInstance>();
        let system_id = **world.resource::<OxrSystemId>();
        let session_config_info = world.non_send_resource::<OxrSessionConfigInfo>();
        let session = world.resource::<OxrSession>();
        let device = world.resource::<RenderDevice>().wgpu_device();

        // TODO!() support other view configurations
        let available_view_configurations = instance.enumerate_view_configurations(system_id)?;
        if !available_view_configurations.contains(&openxr::ViewConfigurationType::PRIMARY_STEREO) {
            return Err(OxrError::NoAvailableViewConfiguration);
        }

        let view_configuration_type = openxr::ViewConfigurationType::PRIMARY_STEREO;

        let view_configuration_views =
            instance.enumerate_view_configuration_views(system_id, view_configuration_type)?;

        let (resolution, _view) = if let Some(resolutions) = &session_config_info.resolutions {
            let mut preferred = None;
            for resolution in resolutions {
                for view_config in view_configuration_views.iter() {
                    if view_config.recommended_image_rect_height == resolution.y
                        && view_config.recommended_image_rect_width == resolution.x
                    {
                        preferred = Some((*resolution, *view_config));
                    }
                }
            }

            if preferred.is_none() {
                for resolution in resolutions {
                    for view_config in view_configuration_views.iter() {
                        if view_config.max_image_rect_height >= resolution.y
                            && view_config.max_image_rect_width >= resolution.x
                        {
                            preferred = Some((*resolution, *view_config));
                        }
                    }
                }
            }

            preferred
        } else {
            view_configuration_views.first().map(|config| {
                (
                    UVec2::new(
                        config.recommended_image_rect_width,
                        config.recommended_image_rect_height,
                    ),
                    *config,
                )
            })
        }
        .ok_or(OxrError::NoAvailableViewConfiguration)?;

        let available_formats = session.enumerate_swapchain_formats()?;

        let format = if let Some(formats) = &session_config_info.formats {
            let mut format = None;
            for wanted_format in formats {
                if available_formats.contains(wanted_format) {
                    format = Some(*wanted_format);
                }
            }
            format
        } else {
            available_formats.first().copied()
        }
        .ok_or(OxrError::NoAvailableFormat)?;

        let swapchain = session.create_swapchain(SwapchainCreateInfo {
            create_flags: SwapchainCreateFlags::EMPTY,
            usage_flags: SwapchainUsageFlags::COLOR_ATTACHMENT | SwapchainUsageFlags::SAMPLED,
            format,
            // TODO() add support for multisampling
            sample_count: 1,
            width: resolution.x,
            height: resolution.y,
            face_count: 1,
            array_size: 2,
            mip_count: 1,
        })?;

        let images = swapchain.enumerate_images(device, format, resolution)?;

        let available_blend_modes =
            instance.enumerate_environment_blend_modes(system_id, view_configuration_type)?;

        // blend mode selection
        let blend_mode = if let Some(wanted_blend_modes) = &session_config_info.blend_modes {
            let mut blend_mode = None;
            for wanted_blend_mode in wanted_blend_modes {
                if available_blend_modes.contains(wanted_blend_mode) {
                    blend_mode = Some(*wanted_blend_mode);
                    break;
                }
            }
            blend_mode
        } else {
            available_blend_modes.first().copied()
        }
        .ok_or(OxrError::NoAvailableBackend)?;

        let graphics_info = OxrGraphicsInfo {
            blend_mode,
            resolution,
            format,
        };

        self.swapchain = Some(swapchain);
        self.images = Some(images);
        self.graphics_info = Some(graphics_info);

        Ok(())
    }

    fn insert_to_world(&mut self, world: &mut World) {
        world.insert_resource(self.graphics_info.clone().unwrap());
        world.insert_resource(self.images.clone().unwrap());
    }

    fn insert_to_render_world(&mut self, world: &mut World) {
        world.insert_resource(self.graphics_info.take().unwrap());
        world.insert_resource(self.images.take().unwrap());
        world.insert_resource(self.swapchain.take().unwrap());
    }

    fn remove_from_world(&mut self, world: &mut World) {
        world.remove_resource::<OxrGraphicsInfo>();
        world.remove_resource::<OxrSwapchainImages>();
    }

    fn remove_from_render_world(&mut self, world: &mut World) {
        world.remove_resource::<OxrGraphicsInfo>();
        world.remove_resource::<OxrSwapchainImages>();
        world.remove_resource::<OxrSwapchain>();
    }
}

pub const XR_TEXTURE_INDEX: u32 = 3383858418;

// TODO: have cameras initialized externally and then recieved by this function.
/// This is needed to properly initialize the texture views so that bevy will set them to the correct resolution despite them being updated in the render world.
pub fn init_views(
    graphics_info: Res<OxrGraphicsInfo>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    swapchain_images: Res<OxrSwapchainImages>,
    root: Query<Entity, With<OxrTrackingRoot>>,
    mut commands: Commands,
) {
    let _span = info_span!("xr_init_views");
    let temp_tex = swapchain_images.first().unwrap();
    // this for loop is to easily add support for quad or mono views in the future.
    let mut views = Vec::with_capacity(2);
    for index in 0..2 {
        info!("{}", graphics_info.resolution);
        let view_handle =
            add_texture_view(&mut manual_texture_views, temp_tex, &graphics_info, index);

        let cam = commands
            .spawn((
                XrCameraBundle {
                    camera: Camera {
                        target: RenderTarget::TextureView(view_handle),
                        ..Default::default()
                    },
                    view: XrCamera(index),
                    ..Default::default()
                },
                // OpenXrTracker,
                // XrRoot::default(),
            ))
            .id();
        match root.get_single() {
            Ok(root) => {
                commands.entity(root).add_child(cam);
            }
            Err(QuerySingleError::NoEntities(_)) => {
                warn!("No OxrTrackingRoot!");
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                warn!("Multiple OxrTrackingRoots! this is not allowed");
            }
        }

        views.push(default());
    }
    commands.insert_resource(OxrViews(views));
}

pub fn wait_frame(mut frame_waiter: ResMut<OxrFrameWaiter>, mut commands: Commands) {
    let _span = info_span!("xr_wait_frame");
    let state = frame_waiter.wait().expect("Failed to wait frame");
    // Here we insert the predicted display time for when this frame will be displayed.
    // TODO: don't add predicted_display_period if pipelined rendering plugin not enabled
    commands.insert_resource(OxrTime(state.predicted_display_time));
}

pub fn locate_views(
    session: Res<OxrSession>,
    ref_space: Res<OxrPrimaryReferenceSpace>,
    time: Res<OxrTime>,
    mut openxr_views: ResMut<OxrViews>,
) {
    let _span = info_span!("xr_locate_views");
    let (flags, xr_views) = session
        .locate_views(
            openxr::ViewConfigurationType::PRIMARY_STEREO,
            **time,
            &ref_space,
        )
        .expect("Failed to locate views");
    match (
        flags & ViewStateFlags::ORIENTATION_VALID == ViewStateFlags::ORIENTATION_VALID,
        flags & ViewStateFlags::POSITION_VALID == ViewStateFlags::POSITION_VALID,
    ) {
        (true, true) => *openxr_views = OxrViews(xr_views),
        (true, false) => {
            for (i, view) in openxr_views.iter_mut().enumerate() {
                let Some(xr_view) = xr_views.get(i) else {
                    break;
                };
                view.pose.orientation = xr_view.pose.orientation;
            }
        }
        (false, true) => {
            for (i, view) in openxr_views.iter_mut().enumerate() {
                let Some(xr_view) = xr_views.get(i) else {
                    break;
                };
                view.pose.position = xr_view.pose.position;
            }
        }
        (false, false) => {}
    }
}

pub fn update_views(
    mut query: Query<(&mut Transform, &mut XrProjection, &XrCamera)>,
    views: ResMut<OxrViews>,
) {
    for (mut transform, mut projection, camera) in query.iter_mut() {
        let Some(view) = views.get(camera.0 as usize) else {
            continue;
        };

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

pub fn update_views_render_world(
    views: Res<OxrViews>,
    root: Res<OxrRootTransform>,
    mut query: Query<(&mut ExtractedView, &XrCamera)>,
) {
    for (mut extracted_view, camera) in query.iter_mut() {
        let Some(view) = views.get(camera.0 as usize) else {
            continue;
        };
        let mut transform = Transform::IDENTITY;
        let openxr::Quaternionf { x, y, z, w } = view.pose.orientation;
        let rotation = Quat::from_xyzw(x, y, z, w);
        transform.rotation = rotation;
        let openxr::Vector3f { x, y, z } = view.pose.position;
        let translation = Vec3::new(x, y, z);
        transform.translation = translation;
        extracted_view.transform = root.0.mul_transform(transform);
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

/// # Safety
/// Images inserted into texture views here should not be written to until [`wait_image`] is ran
pub fn insert_texture_views(
    swapchain_images: Res<OxrSwapchainImages>,
    mut swapchain: ResMut<OxrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    graphics_info: Res<OxrGraphicsInfo>,
) {
    let _span = info_span!("xr_insert_texture_views");
    let index = swapchain.acquire_image().expect("Failed to acquire image");
    let image = &swapchain_images[index as usize];

    for i in 0..2 {
        add_texture_view(&mut manual_texture_views, image, &graphics_info, i);
    }
}

pub fn wait_image(mut swapchain: ResMut<OxrSwapchain>) {
    swapchain
        .wait_image(openxr::Duration::INFINITE)
        .expect("Failed to wait image");
}

pub fn add_texture_view(
    manual_texture_views: &mut ManualTextureViews,
    texture: &wgpu::Texture,
    info: &OxrGraphicsInfo,
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

pub fn begin_frame(mut frame_stream: ResMut<OxrFrameStream>) {
    frame_stream.begin().expect("Failed to begin frame")
}

pub fn release_image(mut swapchain: ResMut<OxrSwapchain>) {
    let _span = info_span!("xr_release_image");
    #[cfg(target_os = "android")]
    {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread_as_daemon();
    }
    swapchain.release_image().unwrap();
}

pub fn end_frame(world: &mut World) {
    let _span = info_span!("xr_end_frame");
    #[cfg(target_os = "android")]
    {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread_as_daemon();
    }
    world.resource_scope::<OxrFrameStream, ()>(|world, mut frame_stream| {
        let mut layers = vec![];
        for layer in world.resource::<OxrRenderLayers>().iter() {
            if let Some(comp_layer) = layer.get(world) {
                layers.push(comp_layer);
            }
        }
        let layers: Vec<_> = layers.iter().map(Box::as_ref).collect();
        frame_stream
            .end(
                **world.resource::<OxrTime>(),
                world.resource::<OxrGraphicsInfo>().blend_mode,
                &layers,
            )
            .expect("Failed to end frame");
    });
}
