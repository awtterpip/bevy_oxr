mod graphics;
pub mod input;
pub mod resource_macros;
pub mod resources;

use std::sync::{Arc, Mutex};

use bevy::app::PluginGroupBuilder;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews};
use bevy::render::renderer::{RenderAdapterInfo, RenderAdapter, RenderDevice, RenderQueue};
use bevy::render::settings::RenderSettings;
use bevy::render::{Render, RenderApp, RenderPlugin, RenderSet};
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use input::XrInput;
use openxr as xr;
use resources::*;
use wgpu::Instance;

const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;

pub const LEFT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(1208214591);
pub const RIGHT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(3383858418);

/// Adds OpenXR support to an App
#[derive(Default)]
pub struct OpenXrPlugin;

#[derive(Resource)]
pub struct FutureXrResources(
    pub  Arc<
        Mutex<
            Option<(
                XrInstance,
                XrSession,
                XrEnvironmentBlendMode,
                XrSessionRunning,
                XrFrameWaiter,
                XrSwapchain,
                XrInput,
                XrViews,
                XrFrameState,
            )>,
        >,
    >,
);

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        let future_xr_resources_wrapper = Arc::new(Mutex::new(None));
        app.insert_resource(FutureXrResources(future_xr_resources_wrapper.clone()));

        let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
            SystemState::new(&mut app.world);
        let primary_window = system_state.get(&app.world).get_single().ok().cloned();

        let (
            device,
            queue,
            adapter_info,
            render_adapter,
            instance,
            xr_instance,
            session,
            blend_mode,
            session_running,
            frame_waiter,
            swapchain,
            input,
            views,
            frame_state,
        ) = graphics::initialize_xr_graphics(primary_window).unwrap();
        debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
        debug!("Configured wgpu adapter Features: {:#?}", device.features());
        let mut future_xr_resources_inner = future_xr_resources_wrapper.lock().unwrap();
        *future_xr_resources_inner = Some((
            xr_instance,
            session,
            blend_mode,
            session_running,
            frame_waiter,
            swapchain,
            input,
            views,
            frame_state,
        ));
        app.add_plugins(DefaultPlugins.set(RenderPlugin { render_settings: RenderSettings::Manual(device, queue, adapter_info, render_adapter, Mutex::new(instance))}));
    }

    fn ready(&self, app: &App) -> bool {
        app.world
            .get_resource::<FutureXrResources>()
            .and_then(|frr| frr.0.try_lock().map(|locked| locked.is_some()).ok())
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        if let Some(future_renderer_resources) = app.world.remove_resource::<FutureXrResources>() {
            let (
                xr_instance,
                session,
                blend_mode,
                session_running,
                frame_waiter,
                swapchain,
                input,
                views,
                frame_state,
            ) = future_renderer_resources.0.lock().unwrap().take().unwrap();

            app.insert_resource(xr_instance.clone())
                .insert_resource(session.clone())
                .insert_resource(blend_mode.clone())
                .insert_resource(session_running.clone())
                .insert_resource(frame_waiter.clone())
                .insert_resource(swapchain.clone())
                .insert_resource(input.clone())
                .insert_resource(views.clone())
                .insert_resource(frame_state.clone());

            let swapchain_mut = swapchain.lock().unwrap();
            let (left, right) = swapchain_mut.get_render_views();
            let format = swapchain_mut.format();
            let left = ManualTextureView {
                texture_view: left.into(),
                size: swapchain_mut.resolution(),
                format,
            };
            let right = ManualTextureView {
                texture_view: right.into(),
                size: swapchain_mut.resolution(),
                format,
            };
            let mut manual_texture_views = app.world.resource_mut::<ManualTextureViews>();
            manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
            manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
            drop(manual_texture_views);
            drop(swapchain_mut);
            let render_app = app.sub_app_mut(RenderApp);

            render_app
                .insert_resource(xr_instance)
                .insert_resource(session)
                .insert_resource(blend_mode)
                .insert_resource(session_running)
                .insert_resource(frame_waiter)
                .insert_resource(swapchain)
                .insert_resource(input)
                .insert_resource(views)
                .insert_resource(frame_state);

            render_app.add_systems(
                Render,
                (
                    pre_frame.in_set(RenderSet::Prepare).before(post_frame),
                    post_frame.in_set(RenderSet::Prepare),
                    post_queue_submit.in_set(RenderSet::Cleanup),
                ),
            );
        }
    }
}

pub struct DefaultXrPlugins;

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group.add(OpenXrPlugin);
        group
    }
}

pub fn pre_frame(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    session_running: Res<XrSessionRunning>,
    frame_state: Res<XrFrameState>,
    frame_waiter: Res<XrFrameWaiter>,
    swapchain: Res<XrSwapchain>,
    xr_input: Res<XrInput>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
) {
    while let Some(event) = instance.poll_event(&mut Default::default()).unwrap() {
        use xr::Event::*;
        match event {
            SessionStateChanged(e) => {
                // Session state change is where we can begin and end sessions, as well as
                // find quit messages!
                info!("entered XR state {:?}", e.state());
                match e.state() {
                    xr::SessionState::READY => {
                        session.begin(VIEW_TYPE).unwrap();
                        session_running.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                    xr::SessionState::STOPPING => {
                        session.end().unwrap();
                        session_running.store(false, std::sync::atomic::Ordering::Relaxed);
                    }
                    xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => return,
                    _ => {}
                }
            }
            InstanceLossPending(_) => return,
            EventsLost(e) => {
                warn!("lost {} XR events", e.lost_event_count());
            }
            _ => {}
        }
    }
    if !session_running.load(std::sync::atomic::Ordering::Relaxed) {
        // Don't grind up the CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
        return;
    }

    *frame_state.lock().unwrap() = Some(frame_waiter.lock().unwrap().wait().unwrap());

    let mut swapchain = swapchain.lock().unwrap();

    swapchain.begin().unwrap();
    swapchain.update_render_views();
    let (left, right) = swapchain.get_render_views();
    let active_action_set = xr::ActiveActionSet::new(&xr_input.action_set);
    match session.sync_actions(&[active_action_set]) {
        Err(err) => {
            eprintln!("{}", err);
        }
        _ => {}
    }
    let format = swapchain.format();
    let left = ManualTextureView {
        texture_view: left.into(),
        size: swapchain.resolution(),
        format,
    };
    let right = ManualTextureView {
        texture_view: right.into(),
        size: swapchain.resolution(),
        format,
    };
    manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
    manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
}

pub fn post_frame(
    views: Res<XrViews>,
    input: Res<XrInput>,
    session: Res<XrSession>,
    xr_frame_state: Res<XrFrameState>,
) {
    *views.lock().unwrap() = session
        .locate_views(
            VIEW_TYPE,
            xr_frame_state
                .lock()
                .unwrap()
                .unwrap()
                .predicted_display_time,
            &input.stage,
        )
        .unwrap()
        .1;
}

pub fn post_queue_submit(
    xr_frame_state: Res<XrFrameState>,
    views: Res<XrViews>,
    input: Res<XrInput>,
    swapchain: Res<XrSwapchain>,
    environment_blend_mode: Res<XrEnvironmentBlendMode>,
) {
    let xr_frame_state = xr_frame_state.lock().unwrap().unwrap();
    let views = &*views.lock().unwrap();
    let stage = &input.stage;
    swapchain
        .lock()
        .unwrap()
        .post_queue_submit(xr_frame_state, views, stage, **environment_blend_mode)
        .unwrap();
}
