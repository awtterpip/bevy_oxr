pub mod graphics;
pub mod input;
// pub mod passthrough;
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
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::render::renderer::{render_system, RenderInstance};
use bevy::render::settings::RenderCreation;
use bevy::render::{Render, RenderApp, RenderPlugin, RenderSet};
use bevy::window::{PresentMode, PrimaryWindow, RawHandleWrapper};
use eyre::anyhow;
use graphics::extensions::XrExtensions;
use graphics::{XrAppInfo, XrPreferdBlendMode};
use input::XrInput;
use openxr as xr;
// use passthrough::{start_passthrough, supports_passthrough, XrPassthroughLayer};
use resources::*;
use xr::{FormFactor, FrameWaiter};
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
        #[cfg(not(target_arch = "wasm32"))]
        match graphics::try_full_init(
            &mut app.world,
            self.reqeusted_extensions.clone(),
            self.prefered_blend_mode,
            self.app_info.clone(),
        ) {
            Ok((device, queue, adapter_info, render_adapter, instance)) => {
                debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
                debug!("Configured wgpu adapter Features: {:#?}", device.features());
                warn!("Starting in Xr");
                app.insert_resource(ActionSets(vec![]));
                app.add_plugins(RenderPlugin {
                    render_creation: RenderCreation::Manual(
                        device,
                        queue,
                        adapter_info,
                        render_adapter,
                        instance,
                    ),
                });
                app.add_plugins(ExtractResourcePlugin::<XrEnableStatus>::default());
                app.insert_resource(XrEnableStatus::Enabled);
            }
            Err(err) => {
                warn!("OpenXR Failed to initialize: {}", err);
                app.add_plugins(RenderPlugin::default());
                app.add_plugins(ExtractResourcePlugin::<XrEnableStatus>::default());
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

    fn finish(&self, app: &mut App) {
        // TODO: Split this up into the indevidual resources
        if app.world.get_resource::<XrEnableStatus>() == Some(&XrEnableStatus::Enabled) {
            warn!("finished xr init");
            let xr_instance = app
                .world
                .get_resource::<XrInstance>()
                .expect("should exist");
            let xr_session = app.world.get_resource::<XrSession>().expect("should exist");
            let hands = xr_instance.exts().ext_hand_tracking.is_some()
                && xr_instance
                    .supports_hand_tracking(
                        xr_instance
                            .system(FormFactor::HEAD_MOUNTED_DISPLAY)
                            .unwrap(),
                    )
                    .is_ok_and(|v| v);
            if hands {
                app.insert_resource(HandTrackingData::new(xr_session).unwrap());
            } else {
                app.insert_resource(DisableHandTracking::Both);
            }
            let xr_swapchain = app
                .world
                .get_resource::<XrSwapchain>()
                .expect("should exist");
            let xr_resolution = app
                .world
                .get_resource::<XrResolution>()
                .expect("should exist");
            let xr_format = app.world.get_resource::<XrFormat>().expect("should exist");

            let (left, right) = xr_swapchain.get_render_views();
            let left = ManualTextureView {
                texture_view: left.into(),
                size: **xr_resolution,
                format: **xr_format,
            };
            let right = ManualTextureView {
                texture_view: right.into(),
                size: **xr_resolution,
                format: **xr_format,
            };
            app.add_systems(PreUpdate, xr_begin_frame.run_if(xr_only()));
            let mut manual_texture_views = app.world.resource_mut::<ManualTextureViews>();
            manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
            manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
            drop(manual_texture_views);
            let render_app = app.sub_app_mut(RenderApp);
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
            .add(XrResourcePlugin)
            // .add(xr_init::RenderRestartPlugin)
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

pub fn xr_begin_frame(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    session_running: Res<XrSessionRunning>,
    mut frame_state: ResMut<XrFrameState>,
    mut frame_waiter: ResMut<XrFrameWaiter>,
    swapchain: Res<XrSwapchain>,
    mut views: ResMut<XrViews>,
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
        *frame_state = match frame_waiter.wait() {
            Ok(a) => a.into(),
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
        **views = session
            .locate_views(VIEW_TYPE, frame_state.predicted_display_time, &input.stage)
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
    xr_frame_state: Option<Res<XrFrameState>>,
    views: Option<Res<XrViews>>,
    input: Option<Res<XrInput>>,
    swapchain: Option<Res<XrSwapchain>>,
    resolution: Option<Res<XrResolution>>,
    environment_blend_mode: Option<Res<XrEnvironmentBlendMode>>,
    // passthrough_layer: Option<Res<XrPassthroughLayer>>,
) {
    let xr_frame_state = xr_frame_state.unwrap();
    let views = views.unwrap();
    let input = input.unwrap();
    let swapchain = swapchain.unwrap();
    let resolution = resolution.unwrap();
    let environment_blend_mode = environment_blend_mode.unwrap();
    {
        let _span = info_span!("xr_release_image").entered();
        swapchain.release_image().unwrap();
    }
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
