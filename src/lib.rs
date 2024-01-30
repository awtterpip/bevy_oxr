pub mod graphics;
pub mod input;
pub mod passthrough;
pub mod resource_macros;
pub mod resources;
pub mod xr_init;
pub mod xr_input;

use std::sync::{Arc, Mutex};

use crate::xr_init::RenderRestartPlugin;
use crate::xr_input::hands::hand_tracking::DisableHandTracking;
use crate::xr_input::oculus_touch::ActionSets;
use bevy::app::{AppExit, PluginGroupBuilder};
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
use passthrough::{start_passthrough, supports_passthrough, Passthrough, PassthroughLayer};
use resources::*;
use xr::FormFactor;
use xr_init::{xr_only, XrEnableStatus, XrRenderData};
use xr_input::controllers::XrControllerType;
use xr_input::hands::emulated::HandEmulationPlugin;
use xr_input::hands::hand_tracking::{HandTrackingData, HandTrackingPlugin};
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
                bool,
                XrPassthrough,
                XrPassthroughLayer,
            )>,
        >,
    >,
);
fn mr_test(mut commands: Commands, passthrough_layer: Option<Res<XrPassthroughLayer>>) {
    commands.insert_resource(ClearColor(Color::rgba(0.0, 0.0, 0.0, 0.0)));
}

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
            SystemState::new(&mut app.world);
        let primary_window = system_state.get(&app.world).get_single().ok().cloned();

        #[cfg(not(target_arch = "wasm32"))]
        match graphics::initialize_xr_graphics(
            primary_window.clone(),
            self.reqeusted_extensions.clone(),
            self.prefered_blend_mode,
            self.app_info.clone(),
        ) {
            Ok((
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
            )) => {
                // std::thread::sleep(Duration::from_secs(5));
                debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
                debug!("Configured wgpu adapter Features: {:#?}", device.features());
                app.insert_resource(xr_instance.clone());
                app.insert_resource(session.clone());
                app.insert_resource(blend_mode.clone());
                app.insert_resource(resolution.clone());
                app.insert_resource(format.clone());
                app.insert_resource(session_running.clone());
                app.insert_resource(frame_waiter.clone());
                app.insert_resource(swapchain.clone());
                app.insert_resource(input.clone());
                app.insert_resource(views.clone());
                app.insert_resource(frame_state.clone());

                // Check if the fb_passthrough extension is available
                let fb_passthrough_available = xr_instance.exts().fb_passthrough.is_some();
                bevy::log::info!(
                    "From OpenXrPlugin: fb_passthrough_available: {}",
                    fb_passthrough_available
                );
                // Get the system for the head-mounted display
                let hmd_system = xr_instance
                    .system(FormFactor::HEAD_MOUNTED_DISPLAY)
                    .unwrap();
                bevy::log::info!("From OpenXrPlugin: hmd_system: {:?}", hmd_system);

                // Check if the system supports passthrough
                let passthrough_supported =
                    supports_passthrough(&xr_instance, hmd_system).is_ok_and(|v| v);
                bevy::log::info!(
                    "From OpenXrPlugin: passthrough_supported: {}",
                    passthrough_supported
                );

                // The passthrough variable will be true only if both fb_passthrough is available and the system supports passthrough
                let passthrough = fb_passthrough_available && passthrough_supported;
                bevy::log::info!("From OpenXrPlugin: passthrough: {}", passthrough);

                let mut p: Option<XrPassthrough> = None;
                let mut pl: Option<XrPassthroughLayer> = None;
                if passthrough {
                    if let Ok((p, pl)) = start_passthrough(&xr_instance, &session) {
                        let xr_data = XrRenderData {
                            xr_instance,
                            xr_session: session,
                            xr_blend_mode: blend_mode,
                            xr_resolution: resolution,
                            xr_format: format,
                            xr_session_running: session_running,
                            xr_frame_waiter: frame_waiter,
                            xr_swapchain: swapchain,
                            xr_input: input,
                            xr_views: views,
                            xr_frame_state: frame_state,
                            xr_passthrough_active: true,
                            xr_passthrough: XrPassthrough::new(Mutex::new(p)),
                            xr_passthrough_layer: XrPassthroughLayer::new(Mutex::new(pl)),
                        };
                        bevy::log::info!("Passthrough is supported!");
                        app.insert_resource(xr_data);
                        app.insert_resource(ClearColor(Color::rgba(0.0, 0.0, 0.0, 0.0)));
                    }

                    if !app.world.contains_resource::<ClearColor>() {
                        info!("ClearColor!");
                    }
                }
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
                app.insert_resource(XrEnableStatus::Enabled);
            }
            Err(err) => {
                warn!("OpenXR Failed to initialize: {}", err);
                app.add_plugins(RenderPlugin::default());
                app.insert_resource(XrEnableStatus::Disabled);
            }
        }
        // app.add_systems(PreUpdate, mr_test);
        #[cfg(target_arch = "wasm32")]
        {
            app.add_plugins(RenderPlugin::default());
            app.insert_resource(XrEnableStatus::Disabled);
        }
    }

    fn ready(&self, app: &App) -> bool {
        app.world
            .get_resource::<XrEnableStatus>()
            .map(|frr| *frr != XrEnableStatus::Waiting)
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        // TODO: Split this up into the indevidual resources
        if let Some(data) = app.world.get_resource::<XrRenderData>().cloned() {
            let hands = data.xr_instance.exts().ext_hand_tracking.is_some()
                && data
                    .xr_instance
                    .supports_hand_tracking(
                        data.xr_instance
                            .system(FormFactor::HEAD_MOUNTED_DISPLAY)
                            .unwrap(),
                    )
                    .is_ok_and(|v| v);
            if hands {
                app.insert_resource(HandTrackingData::new(&data.xr_session).unwrap());
            } else {
                app.insert_resource(DisableHandTracking::Both);
            }

            let (left, right) = data.xr_swapchain.get_render_views();
            let left = ManualTextureView {
                texture_view: left.into(),
                size: *data.xr_resolution,
                format: *data.xr_format,
            };
            let right = ManualTextureView {
                texture_view: right.into(),
                size: *data.xr_resolution,
                format: *data.xr_format,
            };
            app.add_systems(PreUpdate, xr_begin_frame.run_if(xr_only()));
            let mut manual_texture_views = app.world.resource_mut::<ManualTextureViews>();
            manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
            manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
            drop(manual_texture_views);
            let render_app = app.sub_app_mut(RenderApp);

            render_app.insert_resource(data.xr_instance.clone());
            render_app.insert_resource(data.xr_session.clone());
            render_app.insert_resource(data.xr_blend_mode.clone());
            render_app.insert_resource(data.xr_resolution.clone());
            render_app.insert_resource(data.xr_format.clone());
            render_app.insert_resource(data.xr_session_running.clone());
            render_app.insert_resource(data.xr_frame_waiter.clone());
            render_app.insert_resource(data.xr_swapchain.clone());
            render_app.insert_resource(data.xr_input.clone());
            render_app.insert_resource(data.xr_views.clone());
            render_app.insert_resource(data.xr_frame_state.clone());
            render_app.insert_resource(data.xr_passthrough.clone());
            render_app.insert_resource(data.xr_passthrough_layer.clone());
            render_app.insert_resource(XrEnableStatus::Enabled);
            render_app.add_systems(
                Render,
                (
                    post_frame
                        .run_if(xr_only())
                        .before(render_system)
                        .after(RenderSet::ExtractCommands),
                    end_frame.run_if(xr_only()).after(render_system),
                ),
            );
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
            .disable::<RenderPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .add_before::<RenderPlugin, _>(OpenXrPlugin {
                prefered_blend_mode: self.prefered_blend_mode,
                reqeusted_extensions: self.reqeusted_extensions,
                app_info: self.app_info.clone(),
            })
            .add_after::<OpenXrPlugin, _>(OpenXrInput::new(XrControllerType::OculusTouch))
            .add_before::<OpenXrPlugin, _>(RenderRestartPlugin)
            .add(HandEmulationPlugin)
            .add(HandTrackingPlugin)
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

pub fn xr_begin_frame(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    session_running: Res<XrSessionRunning>,
    frame_state: Res<XrFrameState>,
    frame_waiter: Res<XrFrameWaiter>,
    swapchain: Res<XrSwapchain>,
    views: Res<XrViews>,
    input: Res<XrInput>,
    mut app_exit: EventWriter<AppExit>,
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
                            app_exit.send(AppExit);
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            app_exit.send(AppExit);
                            return;
                        }
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
    passthrough_layer: Option<Res<XrPassthroughLayer>>,
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
        // bevy::log::info!(
        //     "passthrough_layer.is_some(): {:?}",
        //     passthrough_layer.is_some()
        // );

        let result = swapchain.end(
            xr_frame_state.lock().unwrap().predicted_display_time,
            &views.lock().unwrap(),
            &input.stage,
            **resolution,
            **environment_blend_mode,
            passthrough_layer.map(|p| PassthroughLayer(*p.lock().unwrap())),
        );
        match result {
            Ok(_) => {}
            Err(e) => warn!("error: {}", e),
        }
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
