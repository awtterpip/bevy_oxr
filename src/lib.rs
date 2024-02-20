pub mod graphics;
pub mod input;
pub mod passthrough;
pub mod resource_macros;
pub mod resources;
pub mod xr_init;
pub mod xr_input;

use std::sync::atomic::AtomicBool;

use crate::xr_init::{StartXrSession, XrInitPlugin};
use crate::xr_input::oculus_touch::ActionSets;
use bevy::app::{AppExit, PluginGroupBuilder};
use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews};
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::renderer::{render_system, RenderInstance};
use bevy::render::settings::RenderCreation;
use bevy::render::{Render, RenderApp, RenderPlugin, RenderSet};
use bevy::window::{PresentMode, PrimaryWindow, RawHandleWrapper};
use graphics::extensions::XrExtensions;
use graphics::{XrAppInfo, XrPreferdBlendMode};
use input::XrInput;
use openxr as xr;
use passthrough::{PassthroughPlugin, XrPassthroughLayer, XrPassthroughState};
use resources::*;
use xr_init::{
    xr_after_wait_only, xr_only, xr_render_only, CleanupXrData, SetupXrData, XrEarlyInitPlugin,
    XrHasWaited, XrShouldRender, XrStatus,
};
use xr_input::actions::OpenXrActionsPlugin;
use xr_input::controllers::XrControllerType;
use xr_input::hands::emulated::HandEmulationPlugin;
use xr_input::hands::hand_tracking::HandTrackingPlugin;
use xr_input::hands::HandPlugin;
use xr_input::xr_camera::XrCameraPlugin;
use xr_input::OpenXrInput;

const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;

pub const LEFT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(1208214591);
pub const RIGHT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(3383858418);

/// Adds OpenXR support to an App
#[derive(Default)]
pub struct OpenXrPlugin {
    reqeusted_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
}

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(XrSessionRunning::new(AtomicBool::new(false)));
        #[cfg(not(target_arch = "wasm32"))]
        match graphics::initialize_xr_instance(
            SystemState::<Query<&RawHandleWrapper, With<PrimaryWindow>>>::new(&mut app.world)
                .get(&app.world)
                .get_single()
                .ok()
                .cloned(),
            self.reqeusted_extensions.clone(),
            self.prefered_blend_mode,
            self.app_info.clone(),
        ) {
            Ok((
                xr_instance,
                oxr_session_setup_info,
                blend_mode,
                device,
                queue,
                adapter_info,
                render_adapter,
                instance,
            )) => {
                debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
                debug!("Configured wgpu adapter Features: {:#?}", device.features());
                warn!("Starting with OpenXR Instance");
                app.insert_resource(xr_instance.clone());
                app.insert_resource(blend_mode);
                app.insert_resource(ActionSets(vec![]));
                app.insert_resource(xr_instance);
                app.insert_resource(blend_mode);
                app.insert_non_send_resource(oxr_session_setup_info);
                let render_instance = RenderInstance(instance.into());
                app.insert_resource(render_instance.clone());
                app.add_plugins(RenderPlugin {
                    render_creation: RenderCreation::Manual(
                        device,
                        queue,
                        adapter_info,
                        render_adapter,
                        render_instance,
                    ),
                });
                app.insert_resource(XrStatus::Disabled);
                app.world.send_event(StartXrSession);
            }
            Err(err) => {
                warn!("OpenXR Instance Failed to initialize: {}", err);
                app.add_plugins(RenderPlugin::default());
                app.insert_resource(XrStatus::NoInstance);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            app.add_plugins(RenderPlugin::default());
            app.insert_resource(XrStatus::Disabled);
        }
        app.add_systems(
            PreUpdate,
            xr_poll_events.run_if(|status: Res<XrStatus>| *status != XrStatus::NoInstance),
        );
        app.add_systems(
            PreUpdate,
            (
                xr_reset_per_frame_resources,
                xr_wait_frame.run_if(xr_only()),
                // xr_begin_frame.run_if(xr_only()),
                locate_views.run_if(xr_only()),
                apply_deferred,
            )
                .chain()
                .after(xr_poll_events),
        );
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
            Render,
            xr_begin_frame
                .run_if(xr_only())
                .run_if(xr_after_wait_only())
                // .run_if(xr_render_only())
                .after(RenderSet::ExtractCommands)
                .before(xr_pre_frame),
        );
        render_app.add_systems(
            Render,
            xr_pre_frame
                .run_if(xr_only())
                .run_if(xr_after_wait_only())
                .run_if(xr_render_only())
                .before(render_system)
                .after(RenderSet::ExtractCommands),
            // .in_set(RenderSet::Prepare),
        );
        render_app.add_systems(
            Render,
            xr_end_frame
                .run_if(xr_only())
                .run_if(xr_after_wait_only())
                .run_if(xr_render_only())
                .in_set(RenderSet::Cleanup),
        );
        render_app.add_systems(
            Render,
            xr_skip_frame
                .run_if(xr_only())
                .run_if(xr_after_wait_only())
                .run_if(not(xr_render_only()))
                .in_set(RenderSet::Cleanup),
        );
    }
}

fn xr_skip_frame(
    xr_swapchain: Res<XrSwapchain>,
    xr_frame_state: Res<XrFrameState>,
    environment_blend_mode: Res<XrEnvironmentBlendMode>,
) {
    let swapchain: &Swapchain = &xr_swapchain;
    match swapchain {
        Swapchain::Vulkan(swap) => {
            swap.stream
                .lock()
                .unwrap()
                .end(
                    xr_frame_state.predicted_display_time,
                    **environment_blend_mode,
                    &[],
                )
                .unwrap();
        }
    }
}

#[derive(Default)]
pub struct DefaultXrPlugins {
    pub reqeusted_extensions: XrExtensions,
    pub prefered_blend_mode: XrPreferdBlendMode,
    pub app_info: XrAppInfo,
}

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> PluginGroupBuilder {
        DefaultPlugins
            .build()
            .set(TaskPoolPlugin {
                task_pool_options: TaskPoolOptions {
                    compute: TaskPoolThreadAssignmentPolicy {
                        min_threads: 2,
                        max_threads: std::usize::MAX, // unlimited max threads
                        percent: 1.0,                 // this value is irrelevant in this case
                    },
                    // keep the defaults for everything else
                    ..default()
                },
            })
            .disable::<PipelinedRenderingPlugin>()
            .disable::<RenderPlugin>()
            .add_before::<RenderPlugin, _>(OpenXrPlugin {
                prefered_blend_mode: self.prefered_blend_mode,
                reqeusted_extensions: self.reqeusted_extensions,
                app_info: self.app_info.clone(),
            })
            .add(XrInitPlugin)
            .add(OpenXrInput::new(XrControllerType::OculusTouch))
            .add(OpenXrActionsPlugin)
            .add(XrCameraPlugin)
            .add_before::<OpenXrPlugin, _>(XrEarlyInitPlugin)
            .add(HandPlugin)
            .add(HandTrackingPlugin)
            .add(HandEmulationPlugin)
            .add(PassthroughPlugin)
            .add(XrResourcePlugin)
            .set(WindowPlugin {
                #[cfg(not(target_os = "android"))]
                primary_window: Some(Window {
                    transparent: true,
                    present_mode: PresentMode::AutoNoVsync,
                    title: self.app_info.name.clone(),
                    ..default()
                }),
                #[cfg(target_os = "android")]
                primary_window: None, // ?
                #[cfg(target_os = "android")]
                exit_condition: bevy::window::ExitCondition::DontExit,
                #[cfg(target_os = "android")]
                close_when_requested: true,
                ..default()
            })
    }
}

fn xr_reset_per_frame_resources(
    mut should: ResMut<XrShouldRender>,
    mut waited: ResMut<XrHasWaited>,
) {
    **should = false;
    **waited = false;
}

fn xr_poll_events(
    instance: Option<Res<XrInstance>>,
    session: Option<Res<XrSession>>,
    session_running: Res<XrSessionRunning>,
    mut app_exit: EventWriter<AppExit>,
    mut setup_xr: EventWriter<SetupXrData>,
    mut cleanup_xr: EventWriter<CleanupXrData>,
) {
    if let (Some(instance), Some(session)) = (instance, session) {
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
                            info!("Calling Session begin :3");
                            session.begin(VIEW_TYPE).unwrap();
                            setup_xr.send_default();
                            session_running.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        xr::SessionState::STOPPING => {
                            session.end().unwrap();
                            session_running.store(false, std::sync::atomic::Ordering::Relaxed);
                            cleanup_xr.send_default();
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            // app_exit.send(AppExit);
                        }

                        _ => {}
                    }
                }
                InstanceLossPending(_) => {
                    app_exit.send_default();
                }
                EventsLost(e) => {
                    warn!("lost {} XR events", e.lost_event_count());
                }
                _ => {}
            }
        }
    }
}

fn xr_begin_frame(swapchain: Res<XrSwapchain>) {
    let _span = info_span!("xr_begin_frame").entered();
    swapchain.begin().unwrap()
}

pub fn xr_wait_frame(
    world: &mut World,
    // mut frame_state: ResMut<XrFrameState>,
    // mut frame_waiter: ResMut<XrFrameWaiter>,
    // mut should_render: ResMut<XrShouldRender>,
    // mut waited: ResMut<XrHasWaited>,
) {
    let mut frame_waiter = world.get_resource_mut::<XrFrameWaiter>().unwrap();
    {
        let _span = info_span!("xr_wait_frame").entered();

        *world.get_resource_mut::<XrFrameState>().unwrap() = match frame_waiter.wait() {
            Ok(a) => a.into(),
            Err(e) => {
                warn!("error: {}", e);
                return;
            }
        };
        let should_render = world.get_resource::<XrFrameState>().unwrap().should_render;
        // frame_state.predicted_display_time = xr::Time::from_nanos(frame_state.predicted_display_time.as_nanos() + frame_state.predicted_display_period.as_nanos());
        **world.get_resource_mut::<XrShouldRender>().unwrap() = should_render;
        **world.get_resource_mut::<XrHasWaited>().unwrap() = true;
    }
}

pub fn xr_pre_frame(
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

#[allow(clippy::too_many_arguments)]
pub fn xr_end_frame(
    xr_frame_state: Res<XrFrameState>,
    views: Res<XrViews>,
    input: Res<XrInput>,
    swapchain: Res<XrSwapchain>,
    resolution: Res<XrResolution>,
    environment_blend_mode: Res<XrEnvironmentBlendMode>,
    passthrough_layer: Option<Res<XrPassthroughLayer>>,
    passthrough_state: Option<Res<XrPassthroughState>>,
) {
    #[cfg(target_os = "android")]
    {
        let ctx = ndk_context::android_context();
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.unwrap();
        let env = vm.attach_current_thread_as_daemon();
    }

    {
        let _span = info_span!("xr_release_image").entered();
        swapchain.release_image().unwrap();
    }
    {
        let _span = info_span!("xr_end_frame").entered();
        let pass_layer = match passthrough_state.as_deref() {
            Some(XrPassthroughState::Running) => passthrough_layer.as_deref(),
            _ => None,
        };
        let result = swapchain.end(
            xr_frame_state.predicted_display_time,
            &views,
            &input.stage,
            **resolution,
            **environment_blend_mode,
            pass_layer,
        );
        match result {
            Ok(_) => {}
            Err(e) => warn!("error: {}", e),
        }
    }
}

pub fn locate_views(
    mut views: ResMut<XrViews>,
    input: Res<XrInput>,
    session: Res<XrSession>,
    xr_frame_state: Res<XrFrameState>,
) {
    let _span = info_span!("xr_locate_views").entered();
    **views = match session.locate_views(
        VIEW_TYPE,
        xr_frame_state.predicted_display_time,
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
