pub mod graphics;
pub mod input;
// pub mod passthrough;
pub mod resource_macros;
pub mod resources;
pub mod xr_init;
pub mod xr_input;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::net::TcpStream;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::xr_init::{StartXrSession, XrInitPlugin};
use crate::xr_input::hands::hand_tracking::DisableHandTracking;
use crate::xr_input::oculus_touch::ActionSets;
use bevy::app::{AppExit, PluginGroupBuilder};
use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews};
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::renderer::RenderInstance;
use bevy::render::settings::RenderCreation;
use bevy::render::{Render, RenderApp, RenderPlugin, RenderSet};
use bevy::window::{PresentMode, PrimaryWindow, RawHandleWrapper};
use graphics::extensions::XrExtensions;
use graphics::{XrAppInfo, XrPreferdBlendMode};
use input::XrInput;
use openxr as xr;
// use passthrough::{start_passthrough, supports_passthrough, XrPassthroughLayer};
use resources::*;
use xr::FormFactor;
use xr_init::{
    xr_only, xr_render_only, CleanupXrData, XrEarlyInitPlugin, XrShouldRender, XrStatus,
};
use xr_input::controllers::XrControllerType;
use xr_input::hands::emulated::HandEmulationPlugin;
use xr_input::hands::hand_tracking::{HandTrackingData, HandTrackingPlugin};
use xr_input::hands::XrHandPlugins;
use xr_input::xr_camera::XrCameraType;
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
                xr_reset_should_render,
                apply_deferred,
                xr_wait_frame.run_if(xr_only()),
                apply_deferred,
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
                .run_if(xr_render_only())
                .after(RenderSet::ExtractCommands)
                .before(xr_pre_frame),
        );
        render_app.add_systems(
            Render,
            xr_pre_frame
                .run_if(xr_only())
                .run_if(xr_render_only())
                .in_set(RenderSet::Prepare),
        );
        render_app.add_systems(
            Render,
            (locate_views, xr_input::xr_camera::xr_camera_head_sync)
                .chain()
                .run_if(xr_only())
                .run_if(xr_render_only())
                .in_set(RenderSet::Prepare),
        );
        render_app.add_systems(
            Render,
            xr_end_frame
                .run_if(xr_only())
                .run_if(xr_render_only())
                .after(RenderSet::Render),
        );
        render_app.insert_resource(TcpConnection(
            TcpStream::connect("192.168.2.100:6969").unwrap(),
        ));
    }
}

#[derive(Resource)]
struct TcpConnection(TcpStream);

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
            .add_after::<OpenXrPlugin, _>(XrInitPlugin)
            .add_before::<OpenXrPlugin, _>(XrEarlyInitPlugin)
            .add(XrHandPlugins)
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
                primary_window: None,
                #[cfg(target_os = "android")]
                exit_condition: bevy::window::ExitCondition::DontExit,
                #[cfg(target_os = "android")]
                close_when_requested: true,
                ..default()
            })
    }
}

fn xr_reset_should_render(mut should: ResMut<XrShouldRender>) {
    **should = false;
}

fn xr_poll_events(
    instance: Option<Res<XrInstance>>,
    session: Option<Res<XrSession>>,
    session_running: Res<XrSessionRunning>,
    mut app_exit: EventWriter<AppExit>,
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
                            session_running.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        xr::SessionState::STOPPING => {
                            session.end().unwrap();
                            session_running.store(false, std::sync::atomic::Ordering::Relaxed);
                            cleanup_xr.send_default();
                        }
                        xr::SessionState::EXITING | xr::SessionState::LOSS_PENDING => {
                            // app_exit.send(AppExit);
                            return;
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
    mut frame_state: ResMut<XrFrameState>,
    mut frame_waiter: ResMut<XrFrameWaiter>,
    mut should_render: ResMut<XrShouldRender>,
) {
    {
        let _span = info_span!("xr_wait_frame").entered();
        *frame_state = match frame_waiter.wait() {
            Ok(a) => a.into(),
            Err(e) => {
                warn!("error: {}", e);
                return;
            }
        };
        **should_render = frame_state.should_render;
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
        info!("wait image");
        swapchain.wait_image().unwrap();
        info!("waited image");
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

pub fn xr_end_frame(
    xr_frame_state: Res<XrFrameState>,
    views: Res<XrViews>,
    input: Res<XrInput>,
    swapchain: Res<XrSwapchain>,
    resolution: Res<XrResolution>,
    environment_blend_mode: Res<XrEnvironmentBlendMode>,
    mut connection: ResMut<TcpConnection>,
    cams: Query<(&Transform, &XrCameraType)>,
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
    let mut cam = None;
    for (t, c) in &cams {
        if *c == XrCameraType::Xr(xr_input::xr_camera::Eye::Left) {
            cam = Some(*t);
            break;
        }
    }
    let _ = std::writeln!(
        &mut connection.0,
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        views[0].pose.position.x,
        views[0].pose.position.y,
        views[0].pose.position.z,
        views[0].pose.orientation.x,
        views[0].pose.orientation.y,
        views[0].pose.orientation.z,
        views[0].pose.orientation.w,
        cam.unwrap().translation.x,
        cam.unwrap().translation.y,
        cam.unwrap().translation.z,
        cam.unwrap().rotation.x,
        cam.unwrap().rotation.y,
        cam.unwrap().rotation.z,
        cam.unwrap().rotation.w,
    );
    {
        let _span = info_span!("xr_end_frame").entered();
        let result = swapchain.end(
            xr_frame_state.predicted_display_time,
            &views,
            &input.stage,
            **resolution,
            **environment_blend_mode,
            // passthrough_layer.map(|p| p.into_inner()),
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
