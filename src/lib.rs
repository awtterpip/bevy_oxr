pub mod input;
pub mod resource_macros;
pub mod resources;
mod graphics;

use std::sync::{Arc, Mutex};

use bevy::ecs::system::SystemState;
use bevy::prelude::*;
use bevy::render::camera::{ManualTextureViews, ManualTextureView, ManualTextureViewHandle};
use bevy::render::{FutureRendererResources, RenderPlugin, RenderApp, Render, RenderSet};
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use input::XrInput;
use resources::*;
use openxr as xr;

const VIEW_TYPE: xr::ViewConfigurationType = xr::ViewConfigurationType::PRIMARY_STEREO;

pub const LEFT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(1208214591);
pub const RIGHT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(3383858418);

/// Adds OpenXR support to an App
#[derive(Default)]
pub struct OpenXrPlugin;

#[derive(Resource)]
pub struct FutureXrResources(
    pub Arc<
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
            )>,
        >,
    >,
);

impl Plugin for OpenXrPlugin {
    fn build(&self, app: &mut App) {
        let future_renderer_resources_wrapper = Arc::new(Mutex::new(None));
        app.insert_resource(FutureRendererResources(
            future_renderer_resources_wrapper.clone(),
        ));

        let future_xr_resources_wrapper = Arc::new(Mutex::new(None));
        app.insert_resource(FutureXrResources(
            future_xr_resources_wrapper.clone()
        ));

        let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
            SystemState::new(&mut app.world);
        let primary_window = system_state.get(&app.world).get_single().ok().cloned();

        bevy::tasks::IoTaskPool::get()
            .spawn_local(async move {
                let (device, queue, adapter_info, render_adapter, instance, xr_instance, session, blend_mode, session_running, frame_waiter, swapchain, input, views) = graphics::initialize_xr_graphics(primary_window).unwrap();
                debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
                debug!("Configured wgpu adapter Features: {:#?}", device.features());
                let mut future_renderer_resources_inner =
                    future_renderer_resources_wrapper.lock().unwrap();
                *future_renderer_resources_inner =
                    Some((device, queue, adapter_info, render_adapter, instance));
                let mut future_xr_resources_inner = future_xr_resources_wrapper.lock().unwrap();
                *future_xr_resources_inner =
                    Some((xr_instance, session, blend_mode, session_running, frame_waiter, swapchain, input, views));
            })
            .detach();

        app.add_systems(Last, pre_frame);
    }

    fn ready(&self, app: &App) -> bool {
        app.world
            .get_resource::<FutureXrResources>()
            .and_then(|frr| frr.0.try_lock().map(|locked| locked.is_some()).ok())
            .unwrap_or(true)
    }

    fn finish(&self, app: &mut App) {
        if let Some(future_renderer_resources) =
            app.world.remove_resource::<FutureXrResources>()
        {
            let (instance, session, blend_mode, session_running, frame_waiter, swapchain, input, views) =
                future_renderer_resources.0.lock().unwrap().take().unwrap();

            app.insert_resource(instance.clone())
                .insert_resource(session.clone())
                .insert_resource(blend_mode.clone())
                .insert_resource(session_running.clone())
                .insert_resource(frame_waiter.clone())
                .insert_resource(swapchain.clone())
                .insert_resource(input.clone())
                .insert_resource(views.clone());

            let render_app = app.sub_app_mut(RenderApp);

            render_app.insert_resource(instance)
                .insert_resource(session)
                .insert_resource(blend_mode)
                .insert_resource(session_running)
                .insert_resource(frame_waiter)
                .insert_resource(swapchain)
                .insert_resource(input)
                .insert_resource(views);
            render_app.add_systems(Render, (post_frame.in_set(RenderSet::Prepare), post_queue_submit.in_set(RenderSet::Cleanup)));
        }
        
    }
}

pub struct DefaultXrPlugins;

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        DefaultPlugins
            .build()
            .add_before::<RenderPlugin, _>(OpenXrPlugin)
    }
}

pub fn pre_frame(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    session_running: Res<XrSessionRunning>,
    frame_state: Res<XrFrameState>,
    frame_waiter: Res<XrFrameWaiter>,
    swapchain: Res<XrSwapchain>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
){
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
                        return
                    }
                    _ => {}
                }
            }
            InstanceLossPending(_) => {
                return
            }
            EventsLost(e) => {
                warn!("lost {} XR events", e.lost_event_count());
            }
            _ => {}
        }
    }
    if !session_running.load(std::sync::atomic::Ordering::Relaxed) {
        // Don't grind up the CPU
        std::thread::sleep(std::time::Duration::from_millis(10));
        return
    }

    *frame_state.lock().unwrap() = Some(frame_waiter.lock().unwrap().wait().unwrap());

    let mut swapchain = swapchain.lock().unwrap();

    swapchain.begin().unwrap();
    let (left, right) = swapchain.get_render_views();
    let left = ManualTextureView::with_default_format(left.into(), swapchain.resolution());
    let right = ManualTextureView::with_default_format(right.into(), swapchain.resolution());
    manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
    manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
}

pub fn post_frame(
    views: Res<XrViews>,
    input: Res<XrInput>,
    session: Res<XrSession>,
    xr_frame_state: Res<XrFrameState>,
) {
    *views.lock().unwrap() = session.locate_views(
        VIEW_TYPE,
        xr_frame_state.lock().unwrap().unwrap().predicted_display_time,
        &input.stage,
    ).unwrap().1;
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
    swapchain.lock().unwrap().post_queue_submit(xr_frame_state, views, stage, **environment_blend_mode).unwrap();
}