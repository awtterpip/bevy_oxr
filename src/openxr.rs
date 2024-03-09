mod extensions;
pub mod graphics;
pub mod render;
mod resources;
pub mod types;

use bevy::ecs::schedule::common_conditions::resource_equals;
use bevy::ecs::system::{Res, ResMut};
use bevy::math::{uvec2, UVec2};
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::renderer::{RenderAdapter, RenderAdapterInfo, RenderInstance, RenderQueue};
use bevy::render::settings::RenderCreation;
use bevy::render::{RenderApp, RenderPlugin};
use bevy::utils::default;
use bevy::DefaultPlugins;
pub use resources::*;
pub use types::*;

use bevy::app::{App, First, Plugin, PluginGroup};
use bevy::log::{error, info, warn};

pub fn xr_entry() -> Result<XrEntry> {
    #[cfg(windows)]
    let entry = openxr::Entry::linked();
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(XrEntry(entry))
}

pub struct XrInitPlugin {
    /// Information about the app this is being used to build.
    pub app_info: AppInfo,
    /// Extensions wanted for this session.
    // TODO!() This should be changed to take a simpler list of features wanted that this crate supports. i.e. hand tracking
    pub exts: XrExtensions,
    /// List of blend modes the openxr session can use. If [None], pick the first available blend mode.
    pub blend_modes: Option<Vec<EnvironmentBlendMode>>,
    /// List of backends the openxr session can use. If [None], pick the first available backend.
    pub backends: Option<Vec<GraphicsBackend>>,
    /// List of formats the openxr session can use. If [None], pick the first available format
    pub formats: Option<Vec<wgpu::TextureFormat>>,
    /// List of resolutions that the openxr swapchain can use. If [None] pick the first available resolution.
    pub resolutions: Option<Vec<UVec2>>,
    /// Passed into the render plugin when added to the app.
    pub synchronous_pipeline_compilation: bool,
}

impl Plugin for XrInitPlugin {
    fn build(&self, app: &mut App) {
        if let Err(e) = init_xr(self, app) {
            panic!("Encountered an error while trying to initialize XR: {e}");
        }
        app.add_systems(First, poll_events);
    }
}

fn init_xr(config: &XrInitPlugin, app: &mut App) -> Result<()> {
    let entry = xr_entry()?;

    let available_exts = entry.enumerate_extensions()?;

    // check available extensions and send a warning for any wanted extensions that aren't available.
    for ext in available_exts.unavailable_exts(&config.exts) {
        error!(
            "Extension \"{ext}\" not available in the current openxr runtime. Disabling extension."
        );
    }

    let available_backends = GraphicsBackend::available_backends(&available_exts);

    // Backend selection
    let backend = if let Some(wanted_backends) = &config.backends {
        let mut backend = None;
        for wanted_backend in wanted_backends {
            if available_backends.contains(wanted_backend) {
                backend = Some(*wanted_backend);
                break;
            }
        }
        backend
    } else {
        available_backends.first().copied()
    }
    .ok_or(XrError::NoAvailableBackend)?;

    let exts = config.exts.clone() & available_exts;

    let instance = entry.create_instance(config.app_info.clone(), exts, backend)?;
    let instance_props = instance.properties()?;

    info!(
        "Loaded OpenXR runtime: {} {}",
        instance_props.runtime_name, instance_props.runtime_version
    );

    let system_id = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;
    let system_props = instance.system_properties(system_id)?;

    info!(
        "Using system: {}",
        if system_props.system_name.is_empty() {
            "<unnamed>"
        } else {
            &system_props.system_name
        }
    );

    // TODO!() support other view configurations
    let available_view_configurations = instance.enumerate_view_configurations(system_id)?;
    if !available_view_configurations.contains(&openxr::ViewConfigurationType::PRIMARY_STEREO) {
        return Err(XrError::NoAvailableViewConfiguration);
    }

    let view_configuration_type = openxr::ViewConfigurationType::PRIMARY_STEREO;

    let available_blend_modes =
        instance.enumerate_environment_blend_modes(system_id, view_configuration_type)?;

    // blend mode selection
    let blend_mode = if let Some(wanted_blend_modes) = &config.blend_modes {
        let mut blend_mode = None;
        for wanted_blend_mode in wanted_blend_modes {
            if available_blend_modes.contains(wanted_blend_mode) {
                blend_mode = Some(*wanted_blend_mode);
                break;
            }
        }
        blend_mode
    } else {
        available_blend_modes.first().copied()
    }
    .ok_or(XrError::NoAvailableBackend)?;

    let view_configuration_views =
        instance.enumerate_view_configuration_views(system_id, view_configuration_type)?;

    let (resolution, _view) = if let Some(resolutions) = &config.resolutions {
        let mut preferred = None;
        for resolution in resolutions {
            for view_config in view_configuration_views.iter() {
                if view_config.recommended_image_rect_height == resolution.y
                    && view_config.recommended_image_rect_width == resolution.x
                {
                    preferred = Some((*resolution, *view_config));
                }
            }
        }

        if preferred.is_none() {
            for resolution in resolutions {
                for view_config in view_configuration_views.iter() {
                    if view_config.max_image_rect_height >= resolution.y
                        && view_config.max_image_rect_width >= resolution.x
                    {
                        preferred = Some((*resolution, *view_config));
                    }
                }
            }
        }

        preferred
    } else {
        if let Some(config) = view_configuration_views.first() {
            Some((
                uvec2(
                    config.recommended_image_rect_width,
                    config.recommended_image_rect_height,
                ),
                *config,
            ))
        } else {
            None
        }
    }
    .ok_or(XrError::NoAvailableViewConfiguration)?;

    let (WgpuGraphics(device, queue, adapter_info, adapter, wgpu_instance), create_info) =
        instance.init_graphics(system_id)?;

    let (session, frame_waiter, frame_stream) =
        unsafe { instance.create_session(system_id, create_info)? };

    let available_formats = session.enumerate_swapchain_formats()?;

    let format = if let Some(formats) = &config.formats {
        let mut format = None;
        for wanted_format in formats {
            if available_formats.contains(wanted_format) {
                format = Some(*wanted_format);
            }
        }
        format
    } else {
        available_formats.first().copied()
    }
    .ok_or(XrError::NoAvailableFormat)?;

    let mut swapchain = session.create_swapchain(SwapchainCreateInfo {
        create_flags: SwapchainCreateFlags::EMPTY,
        usage_flags: SwapchainUsageFlags::COLOR_ATTACHMENT | SwapchainUsageFlags::SAMPLED,
        format,
        // TODO() add support for multisampling
        sample_count: 1,
        width: resolution.x,
        height: resolution.y,
        face_count: 1,
        array_size: 2,
        mip_count: 1,
    })?;

    let images = swapchain.enumerate_images(&device, format, resolution)?;

    let stage = XrStage(
        session
            .create_reference_space(openxr::ReferenceSpaceType::STAGE, openxr::Posef::IDENTITY)?
            .into(),
    );

    app.add_plugins((
        RenderPlugin {
            render_creation: RenderCreation::manual(
                device.into(),
                RenderQueue(queue.into()),
                RenderAdapterInfo(adapter_info),
                RenderAdapter(adapter.into()),
                RenderInstance(wgpu_instance.into()),
            ),
            synchronous_pipeline_compilation: config.synchronous_pipeline_compilation,
        },
        ExtractResourcePlugin::<XrTime>::default(),
        ExtractResourcePlugin::<XrStatus>::default(),
    ));
    let graphics_info = XrGraphicsInfo {
        blend_mode,
        swapchain_resolution: resolution,
        swapchain_format: format,
    };

    app.insert_resource(instance.clone())
        .insert_resource(session.clone())
        .insert_resource(frame_waiter)
        .insert_resource(images.clone())
        .insert_resource(graphics_info)
        .insert_resource(stage.clone())
        .init_resource::<XrStatus>();
    app.sub_app_mut(RenderApp)
        .insert_resource(instance)
        .insert_resource(session)
        .insert_resource(frame_stream)
        .insert_resource(swapchain)
        .insert_resource(images)
        .insert_resource(graphics_info)
        .insert_resource(stage)
        .init_resource::<XrStatus>();

    Ok(())
}

pub fn session_running() -> impl FnMut(Res<XrStatus>) -> bool {
    resource_equals(XrStatus::Enabled)
}

pub fn poll_events(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut xr_status: ResMut<XrStatus>,
) {
    while let Some(event) = instance.poll_event(&mut Default::default()).unwrap() {
        use openxr::Event::*;
        match event {
            SessionStateChanged(e) => {
                // Session state change is where we can begin and end sessions, as well as
                // find quit messages!
                info!("entered XR state {:?}", e.state());
                match e.state() {
                    openxr::SessionState::READY => {
                        info!("Calling Session begin :3");
                        session
                            .begin(openxr::ViewConfigurationType::PRIMARY_STEREO)
                            .unwrap();
                        *xr_status = XrStatus::Enabled;
                    }
                    openxr::SessionState::STOPPING => {
                        session.end().unwrap();
                        *xr_status = XrStatus::Disabled;
                    }
                    // openxr::SessionState::EXITING => {
                    //     if *exit_type == ExitAppOnSessionExit::Always
                    //         || *exit_type == ExitAppOnSessionExit::OnlyOnExit
                    //     {
                    //         app_exit.send_default();
                    //     }
                    // }
                    // openxr::SessionState::LOSS_PENDING => {
                    //     if *exit_type == ExitAppOnSessionExit::Always {
                    //         app_exit.send_default();
                    //     }
                    //     if *exit_type == ExitAppOnSessionExit::OnlyOnExit {
                    //         start_session.send_default();
                    //     }
                    // }
                    _ => {}
                }
            }
            // InstanceLossPending(_) => {
            //     app_exit.send_default();
            // }
            EventsLost(e) => {
                warn!("lost {} XR events", e.lost_event_count());
            }
            _ => {}
        }
    }
}

pub struct DefaultXrPlugins;

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        DefaultPlugins
            .build()
            .disable::<RenderPlugin>()
            .add_before::<RenderPlugin, _>(XrInitPlugin {
                app_info: default(),
                exts: default(),
                blend_modes: default(),
                backends: default(),
                formats: Some(vec![wgpu::TextureFormat::Rgba8UnormSrgb]),
                resolutions: default(),
                synchronous_pipeline_compilation: default(),
            })
            .add(render::XrRenderPlugin)
    }
}
