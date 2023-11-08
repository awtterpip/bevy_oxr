mod graphics;
pub mod input;
pub mod resource_macros;
pub mod resources;
pub mod xr_input;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::xr_input::oculus_touch::ActionSets;
use bevy::app::PluginGroupBuilder;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews};
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::renderer::{render_system, RenderInstance};
use bevy::render::settings::RenderCreation;
use bevy::render::{Render, RenderApp, RenderPlugin, RenderSet};
use bevy::window::{PresentMode, PrimaryWindow, RawHandleWrapper};
use input::XrInput;
use openxr as xr;
use resources::*;
use xr_input::controllers::XrControllerType;
use xr_input::handtracking::HandTrackingTracker;
use xr_input::OpenXrInput;

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
                XrResolution,
                XrFormat,
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
            resolution,
            format,
            session_running,
            frame_waiter,
            swapchain,
            input,
            views,
            frame_state,
        ) = graphics::initialize_xr_graphics(primary_window).unwrap();
        // std::thread::sleep(Duration::from_secs(5));
        debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
        debug!("Configured wgpu adapter Features: {:#?}", device.features());
        let mut future_xr_resources_inner = future_xr_resources_wrapper.lock().unwrap();
        *future_xr_resources_inner = Some((
            xr_instance,
            session,
            blend_mode,
            resolution,
            format,
            session_running,
            frame_waiter,
            swapchain,
            input,
            views,
            frame_state,
        ));
        app.insert_resource(ActionSets(vec![]));
        app.add_plugins(RenderPlugin {
            render_creation: RenderCreation::Manual(
                device,
                queue,
                adapter_info,
                render_adapter,
                RenderInstance(Arc::new(instance)),
            ),
        });
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
                resolution,
                format,
                session_running,
                frame_waiter,
                swapchain,
                input,
                views,
                frame_state,
            ) = future_renderer_resources.0.lock().unwrap().take().unwrap();

            let action_sets = app.world.resource::<ActionSets>().clone();

            app.insert_resource(xr_instance.clone())
                .insert_resource(session.clone())
                .insert_resource(blend_mode.clone())
                .insert_resource(resolution.clone())
                .insert_resource(format.clone())
                .insert_resource(session_running.clone())
                .insert_resource(frame_waiter.clone())
                .insert_resource(swapchain.clone())
                .insert_resource(input.clone())
                .insert_resource(views.clone())
                .insert_resource(frame_state.clone())
                .insert_resource(action_sets.clone())
                .insert_resource(HandTrackingTracker::new(&session).unwrap());

            let (left, right) = swapchain.get_render_views();
            let left = ManualTextureView {
                texture_view: left.into(),
                size: *resolution,
                format: *format,
            };
            let right = ManualTextureView {
                texture_view: right.into(),
                size: *resolution,
                format: *format,
            };
            app.add_systems(PreUpdate, xr_begin_frame);
            let mut manual_texture_views = app.world.resource_mut::<ManualTextureViews>();
            manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
            manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
            drop(manual_texture_views);
            let render_app = app.sub_app_mut(RenderApp);

            render_app
                .insert_resource(xr_instance)
                .insert_resource(session)
                .insert_resource(blend_mode)
                .insert_resource(resolution)
                .insert_resource(format)
                .insert_resource(session_running)
                .insert_resource(frame_waiter)
                .insert_resource(swapchain)
                .insert_resource(input)
                .insert_resource(views)
                .insert_resource(frame_state)
                .insert_resource(action_sets);

            render_app.add_systems(
                Render,
                (
                    post_frame
                        .before(render_system)
                        .after(RenderSet::ExtractCommands),
                    end_frame.after(render_system),
                ),
            );
        }
    }
}

pub struct DefaultXrPlugins;

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> PluginGroupBuilder {
        DefaultPlugins
            .build()
            .disable::<RenderPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .add_before::<RenderPlugin, _>(OpenXrPlugin)
            .add_after::<OpenXrPlugin, _>(OpenXrInput::new(XrControllerType::OculusTouch))
            .set(WindowPlugin {
                #[cfg(not(target_os = "android"))]
                primary_window: Some(Window {
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                #[cfg(target_os = "android")]
                primary_window: None,
                #[cfg(target_os = "android")]
                exit_condition: bevy::window::ExitCondition::DontExit,
                #[cfg(target_os = "android")]
                close_when_requested: true,
                ..default()
            })
    }
}

pub fn xr_begin_frame(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    session_running: Res<XrSessionRunning>,
    frame_state: Res<XrFrameState>,
    frame_waiter: Res<XrFrameWaiter>,
    swapchain: Res<XrSwapchain>,
    views: Res<XrViews>,
    input: Res<XrInput>,
) {
    {
        let _span = info_span!("xr_poll_events");
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
    }
    {
        let _span = info_span!("xr_wait_frame").entered();
        *frame_state.lock().unwrap() = match frame_waiter.lock().unwrap().wait() {
            Ok(a) => a,
            Err(e) => {
                warn!("error: {}", e);
                return;
            }
        };
    }
    {
        let _span = info_span!("xr_begin_frame").entered();
        swapchain.begin().unwrap()
    }
    {
        let _span = info_span!("xr_locate_views").entered();
        *views.lock().unwrap() = session
            .locate_views(
                VIEW_TYPE,
                frame_state.lock().unwrap().predicted_display_time,
                &input.stage,
            )
            .unwrap()
            .1;
    }
}

pub fn post_frame(
    resolution: Res<XrResolution>,
    format: Res<XrFormat>,
    swapchain: Res<XrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
) {
    {
        let _span = info_span!("xr_acquire_image").entered();
        swapchain.acquire_image().unwrap()
    }
    {
        let _span = info_span!("xr_wait_image").entered();
        swapchain.wait_image().unwrap();
    }
    {
        let _span = info_span!("xr_update_manual_texture_views").entered();
        let (left, right) = swapchain.get_render_views();
        let left = ManualTextureView {
            texture_view: left.into(),
            size: **resolution,
            format: **format,
        };
        let right = ManualTextureView {
            texture_view: right.into(),
            size: **resolution,
            format: **format,
        };
        manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
        manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
    }
}

pub fn end_frame(
    xr_frame_state: Res<XrFrameState>,
    views: Res<XrViews>,
    input: Res<XrInput>,
    swapchain: Res<XrSwapchain>,
    resolution: Res<XrResolution>,
    environment_blend_mode: Res<XrEnvironmentBlendMode>,
) {
    {
        let _span = info_span!("xr_release_image").entered();
        swapchain.release_image().unwrap();
    }
    {
        let _span = info_span!("xr_end_frame").entered();
        swapchain
            .end(
                xr_frame_state.lock().unwrap().predicted_display_time,
                &*views.lock().unwrap(),
                &input.stage,
                **resolution,
                **environment_blend_mode,
            )
            .unwrap();
    }
}

pub fn locate_views(
    views: Res<XrViews>,
    input: Res<XrInput>,
    session: Res<XrSession>,
    xr_frame_state: Res<XrFrameState>,
) {
    let _span = info_span!("xr_locate_views").entered();
    *views.lock().unwrap() = match session.locate_views(
        VIEW_TYPE,
        xr_frame_state.lock().unwrap().predicted_display_time,
        &input.stage,
    ) {
        Ok(this) => this,
        Err(err) => {
            warn!("error: {}", err);
            return;
        }
    }
    .1;
}
