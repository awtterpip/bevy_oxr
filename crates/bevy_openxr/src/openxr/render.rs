use bevy::{
    prelude::*,
    render::{
        camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews, RenderTarget},
        extract_resource::ExtractResourcePlugin,
        pipelined_rendering::PipelinedRenderingPlugin,
        view::{ExtractedView, NoFrustumCulling},
        Render, RenderApp,
    },
    transform::TransformSystem,
};
use bevy_mod_xr::{
    camera::{calculate_projection, Fov, XrCamera, XrProjection, XrViewInit},
    session::{
        XrFirst, XrHandleEvents, XrPreDestroySession, XrRenderSet, XrRootTransform,
        XrSessionCreated,
    },
    spaces::XrPrimaryReferenceSpace,
};
use openxr::ViewStateFlags;

use crate::{init::should_run_frame_loop, resources::*};
use crate::{layer_builder::ProjectionLayer, session::OxrSession};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct OxrRenderBegin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct OxrRenderEnd;

pub struct OxrRenderPlugin {
    pub spawn_cameras: bool,
}

impl Default for OxrRenderPlugin {
    fn default() -> Self {
        Self {
            spawn_cameras: true,
        }
    }
}

impl Plugin for OxrRenderPlugin {
    fn build(&self, app: &mut App) {
        if app.is_plugin_added::<PipelinedRenderingPlugin>() {
            app.init_resource::<Pipelined>();
        }

        app.add_plugins((
            ExtractResourcePlugin::<OxrFrameState>::default(),
            ExtractResourcePlugin::<OxrCurrentSessionConfig>::default(),
            ExtractResourcePlugin::<OxrSwapchainImages>::default(),
            ExtractResourcePlugin::<OxrViews>::default(),
        ))
        .add_systems(XrPreDestroySession, clean_views)
        .add_systems(
            XrFirst,
            (
                wait_frame.run_if(should_run_frame_loop),
                update_cameras.run_if(should_run_frame_loop),
            )
                .chain()
                .in_set(XrHandleEvents::FrameLoop),
        )
        .add_systems(
            XrSessionCreated,
            if self.spawn_cameras {
                init_views::<true>
            } else {
                init_views::<false>
            }
            .in_set(XrViewInit),
        )
        .add_systems(
            PostUpdate,
            (locate_views, update_views)
                .before(TransformSystem::TransformPropagate)
                .chain()
                // .run_if(should_render)
                .run_if(should_run_frame_loop),
        )
        .init_resource::<OxrViews>();

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .add_systems(XrPreDestroySession, clean_views)
            .add_systems(
                Render,
                (
                    begin_frame,
                    insert_texture_views,
                    locate_views,
                    update_views_render_world,
                    wait_image,
                )
                    .chain()
                    .in_set(XrRenderSet::PreRender)
                    .run_if(should_run_frame_loop),
            )
            .add_systems(
                Render,
                (release_image, end_frame)
                    .chain()
                    .run_if(should_run_frame_loop)
                    .in_set(XrRenderSet::PostRender),
            )
            .insert_resource(OxrRenderLayers(vec![Box::new(ProjectionLayer)]));
    }
}

// fn update_rendering(app_world: &mut World, _sub_app: &mut App) {
//     app_world.resource_scope(|world, main_thread_executor: Mut<MainThreadExecutor>| {
//         world.resource_scope(|world, mut render_channels: Mut<RenderAppChannels>| {
//             // we use a scope here to run any main thread tasks that the render world still needs to run
//             // while we wait for the render world to be received.
//             let mut render_app = ComputeTaskPool::get()
//                 .scope_with_executor(true, Some(&*main_thread_executor.0), |s| {
//                     s.spawn(async { render_channels.recv().await });
//                 })
//                 .pop()
//                 .unwrap();

//             if matches!(world.resource::<XrState>(), XrState::Stopping) {
//                 world.run_schedule(XrEndSession);
//             }

//             if matches!(world.resource::<XrState>(), XrState::Exiting { .. }) {
//                 world.run_schedule(XrDestroySession);
//                 render_app.app.world.run_schedule(XrDestroySession);
//             }

//             render_app.extract(world);

//             render_channels.send_blocking(render_app);
//         });
//     });
// }

pub const XR_TEXTURE_INDEX: u32 = 3383858418;

pub fn clean_views(
    mut manual_texture_views: ResMut<ManualTextureViews>,
    mut commands: Commands,
    cam_query: Query<(Entity, &XrCamera)>,
) {
    for (e, cam) in &cam_query {
        manual_texture_views.remove(&ManualTextureViewHandle(XR_TEXTURE_INDEX + cam.0));
        commands.entity(e).despawn();
    }
}

pub fn init_views<const SPAWN_CAMERAS: bool>(
    graphics_info: Res<OxrCurrentSessionConfig>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    swapchain_images: Res<OxrSwapchainImages>,
    mut commands: Commands,
) {
    let temp_tex = swapchain_images.first().unwrap();
    // this for loop is to easily add support for quad or mono views in the future.
    for index in 0..2 {
        let _span = debug_span!("xr_init_view").entered();
        info!("XrCamera resolution: {}", graphics_info.resolution);
        let view_handle =
            add_texture_view(&mut manual_texture_views, temp_tex, &graphics_info, index);
        if SPAWN_CAMERAS {
            commands.spawn((
                Camera {
                    target: RenderTarget::TextureView(view_handle),
                    ..Default::default()
                },
                XrCamera(index),
                Projection::custom(XrProjection::default()),
                NoFrustumCulling,
            ));
        }
    }
}

pub fn wait_frame(mut frame_waiter: ResMut<OxrFrameWaiter>, mut commands: Commands) {
    let state = frame_waiter.wait().expect("Failed to wait frame");
    commands.insert_resource(OxrFrameState(state));
}

pub fn update_cameras(
    frame_state: Res<OxrFrameState>,
    mut cameras: Query<(&mut Camera, &XrCamera)>,
) {
    for (mut camera, xr_camera) in &mut cameras {
        camera.target =
            RenderTarget::TextureView(ManualTextureViewHandle(XR_TEXTURE_INDEX + xr_camera.0));
    }
    if frame_state.is_changed() {
        for (mut camera, _) in &mut cameras {
            camera.is_active = frame_state.should_render
        }
    }
}

pub fn locate_views(
    session: Res<OxrSession>,
    ref_space: Res<XrPrimaryReferenceSpace>,
    frame_state: Res<OxrFrameState>,
    mut openxr_views: ResMut<OxrViews>,
    pipelined: Option<Res<Pipelined>>,
) {
    let time = if pipelined.is_some() {
        openxr::Time::from_nanos(
            frame_state.predicted_display_time.as_nanos()
                + frame_state.predicted_display_period.as_nanos(),
        )
    } else {
        frame_state.predicted_display_time
    };
    let (flags, xr_views) = session
        .locate_views(
            openxr::ViewConfigurationType::PRIMARY_STEREO,
            time,
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
    mut query: Query<(&mut Transform, &mut Projection, &XrCamera)>,
    views: ResMut<OxrViews>,
) {
    for (mut transform, mut projection, camera) in query.iter_mut() {
        let Some(view) = views.get(camera.0 as usize) else {
            continue;
        };
        let projection = match projection.as_mut() {
            Projection::Custom(custom) => custom.get_mut::<XrProjection>().unwrap(),
            _ => unreachable!(),
        };

        let projection_matrix = calculate_projection(
            projection.near,
            Fov {
                angle_left: view.fov.angle_left,
                angle_right: view.fov.angle_right,
                angle_down: view.fov.angle_down,
                angle_up: view.fov.angle_up,
            },
        );
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
    root: Res<XrRootTransform>,
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
        extracted_view.world_from_view = root.0.mul_transform(transform);
    }
}

/// # Safety
/// Images inserted into texture views here should not be written to until [`wait_image`] is ran
pub fn insert_texture_views(
    swapchain_images: Res<OxrSwapchainImages>,
    mut swapchain: ResMut<OxrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
    graphics_info: Res<OxrCurrentSessionConfig>,
) {
    let index = swapchain.acquire_image().expect("Failed to acquire image");
    let image = &swapchain_images[index as usize];

    for i in 0..2 {
        let _span = debug_span!("xr_insert_texture_view").entered();
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
    info: &OxrCurrentSessionConfig,
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
    frame_stream.begin().expect("Failed to begin frame");
}

pub fn release_image(mut swapchain: ResMut<OxrSwapchain>) {
    #[cfg(target_os = "android")]
    {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread_as_daemon();
    }
    let _span = debug_span!("xr_release_image").entered();
    swapchain.release_image().unwrap();
}

pub fn end_frame(world: &mut World) {
    #[cfg(target_os = "android")]
    {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread_as_daemon();
    }
    world.resource_scope::<OxrFrameStream, ()>(|world, mut frame_stream| {
        let mut layers = vec![];
        let frame_state = world.resource::<OxrFrameState>();
        let _span = debug_span!("get layers").entered();
        if frame_state.should_render {
            for layer in world.resource::<OxrRenderLayers>().iter() {
                if let Some(layer) = layer.get(world) {
                    layers.push(layer);
                }
            }
        }
        drop(_span);
        let layers: Vec<_> = layers.iter().map(Box::as_ref).collect();
        let _span = debug_span!("xr_end_frame").entered();
        if let Err(e) = frame_stream.end(
            frame_state.predicted_display_time,
            world.resource::<OxrCurrentSessionConfig>().blend_mode,
            &layers,
        ) {
            error!("Failed to end frame stream: {e}");
        }
    });
}
