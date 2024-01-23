pub mod extensions;
mod vulkan;

use bevy::ecs::query::With;
use bevy::ecs::system::{Query, SystemState};
use bevy::ecs::world::World;
use bevy::render::renderer::{
    RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue,
};
use bevy::window::{PrimaryWindow, RawHandleWrapper};
use wgpu::Instance;

use crate::input::XrInput;
use crate::resources::{
    OXrSessionSetupInfo, XrEnvironmentBlendMode, XrFormat, XrFrameState, XrFrameWaiter, XrInstance,
    XrResolution, XrSession, XrSessionRunning, XrSwapchain, XrViews,
};

use openxr as xr;

use self::extensions::XrExtensions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrPreferdBlendMode {
    Opaque,
    Additive,
    AlphaBlend,
}
impl Default for XrPreferdBlendMode {
    fn default() -> Self {
        Self::Opaque
    }
}

#[derive(Clone, Debug)]
pub struct XrAppInfo {
    pub name: String,
}
impl Default for XrAppInfo {
    fn default() -> Self {
        Self {
            name: "Ambient".into(),
        }
    }
}

pub fn start_xr_session(
    window: Option<RawHandleWrapper>,
    session_setup_data: &OXrSessionSetupInfo,
    xr_instance: &XrInstance,
    render_device: &RenderDevice,
    render_adapter: &RenderAdapter,
    wgpu_instance: &Instance,
) -> eyre::Result<(
    XrSession,
    XrResolution,
    XrFormat,
    XrSessionRunning,
    XrFrameWaiter,
    XrSwapchain,
    XrInput,
    XrViews,
    XrFrameState,
)> {
    vulkan::start_xr_session(
        window,
        session_setup_data,
        xr_instance,
        render_device,
        render_adapter,
        wgpu_instance,
    )
}
pub fn initialize_xr_instance(
    window: Option<RawHandleWrapper>,
    reqeusted_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
) -> eyre::Result<(
    XrInstance,
    OXrSessionSetupInfo,
    XrEnvironmentBlendMode,
    RenderDevice,
    RenderQueue,
    RenderAdapterInfo,
    RenderAdapter,
    Instance,
)> {
    vulkan::initialize_xr_instance(window, reqeusted_extensions, prefered_blend_mode, app_info)
}

pub fn try_full_init(
    world: &mut World,
    reqeusted_extensions: XrExtensions,
    prefered_blend_mode: XrPreferdBlendMode,
    app_info: XrAppInfo,
) -> eyre::Result<(
    RenderDevice,
    RenderQueue,
    RenderAdapterInfo,
    RenderAdapter,
    RenderInstance,
)> {
    let mut system_state: SystemState<Query<&RawHandleWrapper, With<PrimaryWindow>>> =
        SystemState::new(world);
    let primary_window = system_state.get(world).get_single().ok().cloned();
    let (
        xr_instance,
        setup_info,
        blend_mode,
        render_device,
        render_queue,
        render_adapter_info,
        render_adapter,
        wgpu_instance,
    ) = initialize_xr_instance(
        primary_window.clone(),
        reqeusted_extensions,
        prefered_blend_mode,
        app_info,
    )?;
    world.insert_resource(xr_instance);
    world.insert_non_send_resource(setup_info);
    // TODO: move BlendMode the session init?
    world.insert_resource(blend_mode);
    let setup_info = world
        .get_non_send_resource::<OXrSessionSetupInfo>()
        .unwrap();
    let xr_instance = world.get_resource::<XrInstance>().unwrap();

    let (
        xr_session,
        xr_resolution,
        xr_format,
        xr_session_running,
        xr_frame_waiter,
        xr_swapchain,
        xr_input,
        xr_views,
        xr_frame_state,
    ) = start_xr_session(
        primary_window,
        setup_info,
        xr_instance,
        &render_device,
        &render_adapter,
        &wgpu_instance,
    )?;
    world.insert_resource(xr_session);
    world.insert_resource(xr_resolution);
    world.insert_resource(xr_format);
    world.insert_resource(xr_session_running);
    world.insert_resource(xr_frame_waiter);
    world.insert_resource(xr_swapchain);
    world.insert_resource(xr_input);
    world.insert_resource(xr_views);
    world.insert_resource(xr_frame_state);

    Ok((
        render_device,
        render_queue,
        render_adapter_info,
        render_adapter,
        RenderInstance(wgpu_instance.into()),
    ))
}

pub fn xr_entry() -> eyre::Result<xr::Entry> {
    #[cfg(windows)]
    let entry = Ok(xr::Entry::linked());
    #[cfg(not(windows))]
    let entry = unsafe { xr::Entry::load().map_err(|e| eyre::eyre!(e)) };
    entry
}
