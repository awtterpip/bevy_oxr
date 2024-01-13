// Just a lot of code that is meant for something way more complex but hey.
// maybe will work on that soon

use std::sync::Arc;

use bevy::{
    ecs::schedule::{ExecutorKind, ScheduleLabel},
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        renderer::{self, RenderAdapter, RenderAdapterInfo, RenderDevice, RenderQueue},
        settings::WgpuSettings,
    },
    window::{PrimaryWindow, RawHandleWrapper},
};
use wgpu::Instance;

use crate::{
    graphics,
    input::XrInput,
    resources::{
        XrEnvironmentBlendMode, XrFormat, XrFrameState, XrInstance, XrResolution, XrSession,
        XrSessionRunning, XrSwapchain, XrViews,
    },
};

#[derive(Resource, Clone)]
pub struct RenderCreationData {
    pub device: RenderDevice,
    pub queue: RenderQueue,
    pub adapter_info: RenderAdapterInfo,
    pub render_adapter: RenderAdapter,
    pub instance: Arc<Instance>,
}

#[derive(Resource, Clone, ExtractResource)]
pub struct XrRenderData {
    pub xr_instance: XrInstance,
    pub xr_session: XrSession,
    pub xr_blend_mode: XrEnvironmentBlendMode,
    pub xr_resolution: XrResolution,
    pub xr_format: XrFormat,
    pub xr_session_running: XrSessionRunning,
    // pub xr_frame_waiter: XrFrameWaiter,
    pub xr_swapchain: XrSwapchain,
    pub xr_input: XrInput,
    pub xr_views: XrViews,
    pub xr_frame_state: XrFrameState,
}

#[derive(Event, Clone, Copy, Debug)]
pub enum XrEnableRequest {
    TryEnable,
    TryDisable,
}
#[derive(Resource, Event, Copy, Clone, PartialEq, Eq, Reflect, ExtractResource)]
pub enum XrEnableStatus {
    Enabled,
    Disabled,
}

#[derive(Resource, Event, Copy, Clone, PartialEq, Eq, Debug, ExtractResource)]
pub enum XrNextEnabledState {
    Enabled,
    Disabled,
}

pub struct RenderRestartPlugin;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPreSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPrePostSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPostSetup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPreCleanup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrCleanup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPostCleanup;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPreRenderUpdate;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrRenderUpdate;

#[derive(Debug, ScheduleLabel, Clone, Copy, Hash, PartialEq, Eq)]
pub struct XrPostRenderUpdate;

pub fn xr_only() -> impl FnMut(Option<Res<'_, XrEnableStatus>>) -> bool {
    resource_exists_and_equals(XrEnableStatus::Enabled)
}

impl Plugin for RenderRestartPlugin {
    fn build(&self, app: &mut App) {
        info!("build RenderRestartPlugin");
        add_schedules(app);
        app.add_plugins(ExtractResourcePlugin::<XrRenderData>::default())
            .add_event::<XrEnableRequest>()
            .add_event::<XrEnableStatus>()
            .add_systems(PostStartup, setup_xr.run_if(xr_only()))
            .add_systems(
                PostUpdate,
                update_xr_stuff.run_if(on_event::<XrEnableRequest>()),
            )
            .add_systems(XrPreRenderUpdate, decide_next_xr_state)
            .add_systems(XrPostRenderUpdate, clear_events)
            .add_systems(
                XrRenderUpdate,
                (
                    cleanup_xr.run_if(resource_exists_and_equals(XrNextEnabledState::Disabled)),
                    // handle_xr_enable_requests,
                    apply_deferred,
                    setup_xr, /* .run_if(resource_exists_and_equals(XrEnableStatus::Enabled)) */
                )
                    .chain(),
            )
            .add_systems(XrCleanup, cleanup_oxr_session);
    }
}

fn clear_events(mut events: ResMut<Events<XrEnableRequest>>) {
    events.clear();
}

fn add_schedules(app: &mut App) {
    let schedules = [
        Schedule::new(XrPreSetup),
        Schedule::new(XrSetup),
        Schedule::new(XrPrePostSetup),
        Schedule::new(XrPostSetup),
        Schedule::new(XrPreRenderUpdate),
        Schedule::new(XrRenderUpdate),
        Schedule::new(XrPostRenderUpdate),
        Schedule::new(XrPreCleanup),
        Schedule::new(XrCleanup),
        Schedule::new(XrPostCleanup),
    ];
    for mut schedule in schedules {
        schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        schedule.set_apply_final_deferred(true);
        app.add_schedule(schedule);
    }
}

pub fn setup_xr(world: &mut World) {
    world.run_schedule(XrPreSetup);
    world.run_schedule(XrSetup);
    world.run_schedule(XrPrePostSetup);
    world.run_schedule(XrPostSetup);
}
fn cleanup_xr(world: &mut World) {
    world.run_schedule(XrPreCleanup);
    world.run_schedule(XrCleanup);
    world.run_schedule(XrPostCleanup);
}

fn cleanup_oxr_session(xr_status: Option<Res<XrEnableStatus>>, session: Option<ResMut<XrSession>>) {
    if let (Some(XrEnableStatus::Disabled), Some(s)) = (xr_status.map(|v| v.into_inner()), session)
    {
        s.into_inner().request_exit().unwrap();
    }
}

pub fn update_xr_stuff(world: &mut World) {
    world.run_schedule(XrPreRenderUpdate);
    world.run_schedule(XrRenderUpdate);
    world.run_schedule(XrPostRenderUpdate);
}

fn setup_xr_graphics() {}

fn enable_xr() {}

// fn handle_xr_enable_requests(
//     primary_window: Query<&RawHandleWrapper, With<PrimaryWindow>>,
//     mut commands: Commands,
//     next_state: Res<XrNextEnabledState>,
//     on_main: Option<NonSend<ForceMain>>,
// ) {
//     // Just to force this system onto the main thread because of unsafe code
//     let _ = on_main;
//
//     let (creation_data, xr_data) = match next_state.into_inner() {
//         XrNextEnabledState::Enabled => {
//             let (
//                 device,
//                 queue,
//                 adapter_info,
//                 render_adapter,
//                 instance,
//                 xr_instance,
//                 session,
//                 blend_mode,
//                 resolution,
//                 format,
//                 session_running,
//                 frame_waiter,
//                 swapchain,
//                 input,
//                 views,
//                 frame_state,
//             ) = graphics::initialize_xr_graphics(primary_window.get_single().ok().cloned())
//                 .unwrap();
//
//             commands.insert_resource(XrEnableStatus::Enabled);
//             (
//                 RenderCreationData {
//                     device,
//                     queue,
//                     adapter_info,
//                     render_adapter,
//                     instance: Arc::new(instance),
//                 },
//                 Some(XrRenderData {
//                     xr_instance,
//                     xr_session: session,
//                     xr_blend_mode: blend_mode,
//                     xr_resolution: resolution,
//                     xr_format: format,
//                     xr_session_running: session_running,
//                     xr_frame_waiter: frame_waiter,
//                     xr_swapchain: swapchain,
//                     xr_input: input,
//                     xr_views: views,
//                     xr_frame_state: frame_state,
//                 }),
//             )
//         }
//         XrNextEnabledState::Disabled => (
//             init_non_xr_graphics(primary_window.get_single().ok().cloned()),
//             None,
//         ),
//     };
//
//     commands.insert_resource(creation_data.device);
//     commands.insert_resource(creation_data.queue);
//     commands.insert_resource(RenderInstance(creation_data.instance));
//     commands.insert_resource(creation_data.adapter_info);
//     commands.insert_resource(creation_data.render_adapter);
//
//     if let Some(xr_data) = xr_data {
//         // TODO: Remove this lib.rs:144
//         commands.insert_resource(xr_data.clone());
//
//         commands.insert_resource(xr_data.xr_instance);
//         commands.insert_resource(xr_data.xr_session);
//         commands.insert_resource(xr_data.xr_blend_mode);
//         commands.insert_resource(xr_data.xr_resolution);
//         commands.insert_resource(xr_data.xr_format);
//         commands.insert_resource(xr_data.xr_session_running);
//         commands.insert_resource(xr_data.xr_frame_waiter);
//         commands.insert_resource(xr_data.xr_input);
//         commands.insert_resource(xr_data.xr_views);
//         commands.insert_resource(xr_data.xr_frame_state);
//         commands.insert_resource(xr_data.xr_swapchain);
//     } else {
//         commands.remove_resource::<XrRenderData>();
//
//         commands.remove_resource::<XrInstance>();
//         commands.remove_resource::<XrSession>();
//         commands.remove_resource::<XrEnvironmentBlendMode>();
//         commands.remove_resource::<XrResolution>();
//         commands.remove_resource::<XrFormat>();
//         commands.remove_resource::<XrSessionRunning>();
//         commands.remove_resource::<XrFrameWaiter>();
//         commands.remove_resource::<XrSwapchain>();
//         commands.remove_resource::<XrInput>();
//         commands.remove_resource::<XrViews>();
//         commands.remove_resource::<XrFrameState>();
//     }
// }

fn decide_next_xr_state(
    mut commands: Commands,
    mut events: EventReader<XrEnableRequest>,
    xr_status: Option<Res<XrEnableStatus>>,
) {
    info!("hm");
    let request = match events.read().next() {
        Some(v) => v,
        None => return,
    };
    info!("ok");
    match (request, xr_status.as_deref()) {
        (XrEnableRequest::TryEnable, Some(XrEnableStatus::Enabled)) => {
            info!("Xr Already Enabled! ignoring request");
            return;
        }
        (XrEnableRequest::TryDisable, Some(XrEnableStatus::Disabled)) => {
            info!("Xr Already Disabled! ignoring request");
            return;
        }
        // (_, Some(XrEnableStatus::Waiting)) => {
        //     info!("Already Handling Request! ignoring request");
        //     return;
        // }
        _ => {}
    }
    let r = match request {
        XrEnableRequest::TryEnable => XrNextEnabledState::Enabled,
        XrEnableRequest::TryDisable => XrNextEnabledState::Disabled,
    };
    info!("{:#?}", r);
    commands.insert_resource(r);
}

pub fn init_non_xr_graphics(primary_window: Option<RawHandleWrapper>) -> RenderCreationData {
    let settings = WgpuSettings::default();

    let async_renderer = async move {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            // Probably a bad idea unwraping here but on the other hand no backends
            backends: settings.backends.unwrap(),
            dx12_shader_compiler: settings.dx12_shader_compiler.clone(),
        });
        let surface = primary_window.map(|wrapper| unsafe {
            // SAFETY: Plugins should be set up on the main thread.
            let handle = wrapper.get_handle();
            instance
                .create_surface(&handle)
                .expect("Failed to create wgpu surface")
        });

        let request_adapter_options = wgpu::RequestAdapterOptions {
            power_preference: settings.power_preference,
            compatible_surface: surface.as_ref(),
            ..Default::default()
        };

        let (device, queue, adapter_info, render_adapter) =
            renderer::initialize_renderer(&instance, &settings, &request_adapter_options).await;
        debug!("Configured wgpu adapter Limits: {:#?}", device.limits());
        debug!("Configured wgpu adapter Features: {:#?}", device.features());
        RenderCreationData {
            device,
            queue,
            adapter_info,
            render_adapter,
            instance: Arc::new(instance),
        }
    };
    // No need for wasm in bevy_oxr web xr would be a different crate
    futures_lite::future::block_on(async_renderer)
}
