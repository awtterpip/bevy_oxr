use bevy::app::{App, First, Plugin};
use bevy::ecs::event::EventWriter;
use bevy::ecs::schedule::common_conditions::{not, on_event};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Commands, Res, ResMut, Resource};
use bevy::ecs::world::World;
use bevy::log::{error, info};
use bevy::math::{uvec2, UVec2};
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::renderer::{
    RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue,
};
use bevy::render::settings::RenderCreation;
use bevy::render::{ExtractSchedule, MainWorld, RenderApp, RenderPlugin};
use bevy_xr::session::{
    BeginXrSession, CreateXrSession, XrInstanceCreated, XrInstanceDestroyed, XrSessionState,
};

use crate::error::XrError;
use crate::graphics::GraphicsBackend;
use crate::resources::*;
use crate::types::*;

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

pub fn instance_created(status: Option<Res<XrStatus>>) -> bool {
    status.is_some_and(|status| status.instance_created)
}

pub fn session_created(status: Option<Res<XrStatus>>) -> bool {
    status.is_some_and(|status| status.session_created)
}

pub fn session_ready(status: Option<Res<XrStatus>>) -> bool {
    status.is_some_and(|status| status.session_ready)
}

pub fn session_running(status: Option<Res<XrStatus>>) -> bool {
    status.is_some_and(|status| status.session_running)
}

impl Plugin for XrInitPlugin {
    fn build(&self, app: &mut App) {
        if let Err(e) = init_xr(&self, app) {
            error!("Failed to initialize openxr instance: {e}.");
            app.add_plugins(RenderPlugin::default())
                .insert_resource(XrStatus::UNINITIALIZED);
        }
    }
}

fn xr_entry() -> Result<XrEntry> {
    #[cfg(windows)]
    let entry = openxr::Entry::linked();
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(XrEntry(entry))
}

/// This is called from [`XrInitPlugin::build()`]. Its a separate function so that we can return a [Result] and control flow is cleaner.
fn init_xr(config: &XrInitPlugin, app: &mut App) -> Result<()> {
    let entry = xr_entry()?;

    let available_exts = entry.enumerate_extensions()?;

    // check available extensions and send a warning for any wanted extensions that aren't available.
    for ext in available_exts.unavailable_exts(&config.exts) {
        error!(
            "Extension \"{ext}\" not available in the current OpenXR runtime. Disabling extension."
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

    let (WgpuGraphics(device, queue, adapter_info, adapter, wgpu_instance), create_info) =
        instance.init_graphics(system_id)?;

    app.world.send_event(XrInstanceCreated);
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
    ))
    .insert_resource(instance.clone())
    .insert_resource(SystemId(system_id))
    .insert_resource(XrStatus {
        instance_created: true,
        ..Default::default()
    })
    .insert_non_send_resource(XrSessionInitConfig {
        blend_modes: config.blend_modes.clone(),
        formats: config.formats.clone(),
        resolutions: config.resolutions.clone(),
        create_info,
    })
    .add_systems(
        First,
        (
            poll_events.run_if(instance_created),
            create_xr_session
                .run_if(not(session_created))
                .run_if(on_event::<CreateXrSession>()),
            begin_xr_session
                .run_if(session_ready)
                .run_if(on_event::<BeginXrSession>()),
        )
            .chain(),
    )
    .sub_app_mut(RenderApp)
    .insert_resource(instance)
    .insert_resource(SystemId(system_id))
    .add_systems(
        ExtractSchedule,
        transfer_xr_resources.run_if(not(session_running)),
    );

    Ok(())
}

/// This is used to store information from startup that is needed to create the session after the instance has been created.
struct XrSessionInitConfig {
    /// List of blend modes the openxr session can use. If [None], pick the first available blend mode.
    blend_modes: Option<Vec<EnvironmentBlendMode>>,
    /// List of formats the openxr session can use. If [None], pick the first available format
    formats: Option<Vec<wgpu::TextureFormat>>,
    /// List of resolutions that the openxr swapchain can use. If [None] pick the first available resolution.
    resolutions: Option<Vec<UVec2>>,
    /// Graphics info used to create a session.
    create_info: SessionCreateInfo,
}

pub fn create_xr_session(world: &mut World) {
    let Some(create_info) = world.remove_non_send_resource() else {
        error!(
            "Failed to retrive SessionCreateInfo. This is likely due to improper initialization."
        );
        return;
    };

    let Some(instance) = world.get_resource().cloned() else {
        error!("Failed to retrieve XrInstance. This is likely due to improper initialization.");
        return;
    };

    let Some(system_id) = world.get_resource::<SystemId>().cloned() else {
        error!("Failed to retrieve SystemId. THis is likely due to improper initialization");
        return;
    };

    if let Err(e) = create_xr_session_inner(world, instance, *system_id, create_info) {
        error!("Failed to initialize XrSession: {e}");
    }
}

/// This is called from [create_xr_session]. It is a separate function to allow us to return a [Result] and make control flow cleaner.
fn create_xr_session_inner(
    world: &mut World,
    instance: XrInstance,
    system_id: openxr::SystemId,
    config: XrSessionInitConfig,
) -> Result<()> {
    let (session, frame_waiter, frame_stream) =
        unsafe { instance.create_session(system_id, config.create_info)? };

    // TODO!() support other view configurations
    let available_view_configurations = instance.enumerate_view_configurations(system_id)?;
    if !available_view_configurations.contains(&openxr::ViewConfigurationType::PRIMARY_STEREO) {
        return Err(XrError::NoAvailableViewConfiguration);
    }

    let view_configuration_type = openxr::ViewConfigurationType::PRIMARY_STEREO;

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

    let images = swapchain.enumerate_images(
        world.resource::<RenderDevice>().wgpu_device(),
        format,
        resolution,
    )?;

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

    let stage = XrStage(
        session
            .create_reference_space(openxr::ReferenceSpaceType::STAGE, openxr::Posef::IDENTITY)?
            .into(),
    );

    let graphics_info = XrGraphicsInfo {
        blend_mode,
        resolution,
        format,
    };

    world.resource_mut::<XrStatus>().session_created = true;
    world.insert_resource(session.clone());
    world.insert_resource(frame_waiter);
    world.insert_resource(images.clone());
    world.insert_resource(graphics_info.clone());
    world.insert_resource(stage.clone());
    world.insert_resource(frame_stream.clone());
    world.insert_resource(XrRenderResources {
        session,
        frame_stream,
        swapchain,
        images,
        graphics_info,
        stage,
    });

    Ok(())
}

pub fn begin_xr_session(
    session: Res<XrSession>,
    mut session_state: EventWriter<XrSessionState>,
    mut status: ResMut<XrStatus>,
) {
    session
        .begin(openxr::ViewConfigurationType::PRIMARY_STEREO)
        .expect("Failed to begin session");
    status.session_running = true;
    session_state.send(XrSessionState::Running);
}

/// This is used solely to transport resources from the main world to the render world.
#[derive(Resource)]
struct XrRenderResources {
    session: XrSession,
    frame_stream: XrFrameStream,
    swapchain: XrSwapchain,
    images: XrSwapchainImages,
    graphics_info: XrGraphicsInfo,
    stage: XrStage,
}

/// This system transfers important render resources from the main world to the render world when a session is created.
pub fn transfer_xr_resources(mut commands: Commands, mut world: ResMut<MainWorld>) {
    let Some(XrRenderResources {
        session,
        frame_stream,
        swapchain,
        images,
        graphics_info,
        stage,
    }) = world.remove_resource()
    else {
        return;
    };

    commands.insert_resource(session);
    commands.insert_resource(frame_stream);
    commands.insert_resource(swapchain);
    commands.insert_resource(images);
    commands.insert_resource(graphics_info);
    commands.insert_resource(stage);
}

/// Poll any OpenXR events and handle them accordingly
pub fn poll_events(
    instance: Res<XrInstance>,
    session: Option<Res<XrSession>>,
    mut session_state: EventWriter<XrSessionState>,
    mut instance_destroyed: EventWriter<XrInstanceDestroyed>,
    mut status: ResMut<XrStatus>,
) {
    let mut buffer = Default::default();
    while let Some(event) = instance
        .poll_event(&mut buffer)
        .expect("Failed to poll event")
    {
        use openxr::Event::*;
        match event {
            SessionStateChanged(e) => {
                if let Some(ref session) = session {
                    info!("entered XR state {:?}", e.state());
                    use openxr::SessionState;

                    match e.state() {
                        SessionState::IDLE => {
                            status.session_ready = false;
                            session_state.send(XrSessionState::Idle);
                        }
                        SessionState::READY => {
                            status.session_ready = true;
                            session_state.send(XrSessionState::Ready);
                        }
                        SessionState::STOPPING => {
                            status.session_running = false;
                            status.session_ready = false;
                            session.end().expect("Failed to end session");
                            session_state.send(XrSessionState::Stopping);
                        }
                        // TODO: figure out how to destroy the session
                        SessionState::EXITING | SessionState::LOSS_PENDING => {
                            status.session_running = false;
                            status.session_created = false;
                            status.session_ready = false;
                            session.end().expect("Failed to end session");
                            session_state.send(XrSessionState::Destroyed);
                        }
                        _ => {}
                    }
                }
            }
            InstanceLossPending(_) => {
                *status = XrStatus::UNINITIALIZED;
                instance_destroyed.send_default();
            }
            _ => {}
        }
    }
}
