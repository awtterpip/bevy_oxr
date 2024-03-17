use bevy::app::AppExit;
use bevy::math::uvec2;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::renderer::{
    RenderAdapter, RenderAdapterInfo, RenderDevice, RenderInstance, RenderQueue,
};
use bevy::render::settings::RenderCreation;
use bevy::render::{MainWorld, RenderApp, RenderPlugin};
use bevy_xr::session::{
    handle_session, session_available, session_running, status_equals, BeginXrSession,
    CreateXrSession, EndXrSession, XrSharedStatus, XrStatus,
};

use crate::graphics::*;
use crate::resources::*;
use crate::types::*;

pub fn session_started(started: Option<Res<XrSessionStarted>>) -> bool {
    started.is_some_and(|started| started.get())
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum XrPreUpdateSet {
    PollEvents,
    HandleEvents,
}

#[derive(Resource, Clone, Copy, PartialEq)]
pub struct AppExiting(bool);

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
        match self.init_xr() {
            Ok((
                instance,
                system_id,
                WgpuGraphics(device, queue, adapter_info, adapter, wgpu_instance),
                session_create_info,
            )) => {
                let status = XrSharedStatus::new(XrStatus::Available);

                app.add_plugins((
                    RenderPlugin {
                        render_creation: RenderCreation::manual(
                            device.into(),
                            RenderQueue(queue.into()),
                            RenderAdapterInfo(adapter_info),
                            RenderAdapter(adapter.into()),
                            RenderInstance(wgpu_instance.into()),
                        ),
                        synchronous_pipeline_compilation: self.synchronous_pipeline_compilation,
                    },
                    ExtractResourcePlugin::<XrTime>::default(),
                ))
                .add_systems(
                    PreUpdate,
                    (
                        poll_events
                            .run_if(session_available)
                            .in_set(XrPreUpdateSet::PollEvents),
                        (
                            (create_xr_session, apply_deferred)
                                .chain()
                                .run_if(on_event::<CreateXrSession>())
                                .run_if(status_equals(XrStatus::Available)),
                            begin_xr_session
                                .run_if(on_event::<BeginXrSession>())
                                .run_if(status_equals(XrStatus::Ready)),
                            end_xr_session
                                .run_if(on_event::<EndXrSession>())
                                .run_if(status_equals(XrStatus::Exiting)),
                        )
                            .in_set(XrPreUpdateSet::HandleEvents),
                    ),
                )
                .add_systems(
                    Last,
                    app_exit_xr
                        .run_if(resource_equals(AppExiting(false)))
                        .run_if(on_event::<AppExit>()),
                )
                .insert_resource(instance.clone())
                .insert_resource(system_id)
                .insert_resource(status.clone())
                .insert_non_send_resource(session_create_info);

                let render_app = app.sub_app_mut(RenderApp);
                render_app
                    .insert_resource(instance)
                    .insert_resource(system_id)
                    .insert_resource(status)
                    .add_systems(
                        ExtractSchedule,
                        transfer_xr_resources.run_if(not(session_running)),
                    );
            }
            Err(e) => {
                error!("Failed to initialize openxr: {e}");
                let status = XrSharedStatus::new(XrStatus::Unavailable);

                app.add_plugins(RenderPlugin::default())
                    .insert_resource(status.clone());

                let render_app = app.sub_app_mut(RenderApp);

                render_app.insert_resource(status);
            }
        };

        app.configure_sets(
            PreUpdate,
            (
                XrPreUpdateSet::PollEvents.before(handle_session),
                XrPreUpdateSet::HandleEvents.after(handle_session),
            ),
        );

        let session_started = XrSessionStarted::default();

        app.insert_resource(session_started.clone())
            .insert_resource(AppExiting(false));

        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(session_started);
    }
}

fn xr_entry() -> Result<XrEntry> {
    #[cfg(windows)]
    let entry = openxr::Entry::linked();
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(XrEntry(entry))
}

impl XrInitPlugin {
    fn init_xr(&self) -> Result<(XrInstance, XrSystemId, WgpuGraphics, XrSessionCreateInfo)> {
        let entry = xr_entry()?;

        let available_exts = entry.enumerate_extensions()?;

        // check available extensions and send a warning for any wanted extensions that aren't available.
        for ext in available_exts.unavailable_exts(&self.exts) {
            error!(
                "Extension \"{ext}\" not available in the current OpenXR runtime. Disabling extension."
            );
        }

        let available_backends = GraphicsBackend::available_backends(&available_exts);

        // Backend selection
        let backend = if let Some(wanted_backends) = &self.backends {
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

        let exts = self.exts.clone() & available_exts;

        let instance = entry.create_instance(
            self.app_info.clone(),
            exts,
            &["XR_APILAYER_LUNARG_api_dump"],
            // &[],
            backend,
        )?;
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

        let (graphics, graphics_info) = instance.init_graphics(system_id)?;

        let session_create_info = XrSessionCreateInfo {
            blend_modes: self.blend_modes.clone(),
            formats: self.formats.clone(),
            resolutions: self.resolutions.clone(),
            graphics_info,
        };

        Ok((
            instance,
            XrSystemId(system_id),
            graphics,
            session_create_info,
        ))
    }
}

fn init_xr_session(
    device: &wgpu::Device,
    instance: &XrInstance,
    system_id: openxr::SystemId,
    XrSessionCreateInfo {
        blend_modes,
        formats,
        resolutions,
        graphics_info,
    }: XrSessionCreateInfo,
) -> Result<(
    XrSession,
    XrFrameWaiter,
    XrFrameStream,
    XrSwapchain,
    XrSwapchainImages,
    XrGraphicsInfo,
    XrStage,
)> {
    let (session, frame_waiter, frame_stream) =
        unsafe { instance.create_session(system_id, graphics_info)? };

    // TODO!() support other view configurations
    let available_view_configurations = instance.enumerate_view_configurations(system_id)?;
    if !available_view_configurations.contains(&openxr::ViewConfigurationType::PRIMARY_STEREO) {
        return Err(XrError::NoAvailableViewConfiguration);
    }

    let view_configuration_type = openxr::ViewConfigurationType::PRIMARY_STEREO;

    let view_configuration_views =
        instance.enumerate_view_configuration_views(system_id, view_configuration_type)?;

    let (resolution, _view) = if let Some(resolutions) = &resolutions {
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

    let format = if let Some(formats) = &formats {
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

    let images = swapchain.enumerate_images(device, format, resolution)?;

    let available_blend_modes =
        instance.enumerate_environment_blend_modes(system_id, view_configuration_type)?;

    // blend mode selection
    let blend_mode = if let Some(wanted_blend_modes) = &blend_modes {
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

    Ok((
        session,
        frame_waiter,
        frame_stream,
        swapchain,
        images,
        graphics_info,
        stage,
    ))
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

pub fn create_xr_session(
    device: Res<RenderDevice>,
    instance: Res<XrInstance>,
    create_info: NonSend<XrSessionCreateInfo>,
    system_id: Res<XrSystemId>,
    mut commands: Commands,
) {
    match init_xr_session(
        device.wgpu_device(),
        &instance,
        **system_id,
        create_info.clone(),
    ) {
        Ok((session, frame_waiter, frame_stream, swapchain, images, graphics_info, stage)) => {
            commands.insert_resource(session.clone());
            commands.insert_resource(frame_waiter);
            commands.insert_resource(images.clone());
            commands.insert_resource(graphics_info.clone());
            commands.insert_resource(stage.clone());
            commands.insert_resource(frame_stream.clone());
            commands.insert_resource(XrRenderResources {
                session,
                frame_stream,
                swapchain,
                images,
                graphics_info,
                stage,
            });
        }
        Err(e) => error!("Failed to initialize XrSession: {e}"),
    }
}

pub fn begin_xr_session(session: Res<XrSession>, session_started: Res<XrSessionStarted>) {
    session
        .begin(openxr::ViewConfigurationType::PRIMARY_STEREO)
        .expect("Failed to begin session");
    session_started.set(true);
}

pub fn end_xr_session(session: Res<XrSession>, session_started: Res<XrSessionStarted>) {
    session.end().expect("Failed to end session");
    session_started.set(false);
}

pub fn app_exit_xr(
    mut app_exiting: ResMut<AppExiting>,
    mut app_exit_events: ResMut<Events<AppExit>>,
    session_started: Res<XrSessionStarted>,
) {
    // we need to temporarily intercept the exit event to allow the session to exit.
    app_exit_events.clear();
    *app_exiting = AppExiting(true);
    session_started.set(false);
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

/// Polls any OpenXR events and handles them accordingly
pub fn poll_events(
    instance: Res<XrInstance>,
    status: Res<XrSharedStatus>,
    session: Option<Res<XrSession>>,
    app_exiting: Res<AppExiting>,
    mut exit_app: EventWriter<AppExit>,
) {
    let mut buffer = Default::default();
    while let Some(event) = instance
        .poll_event(&mut buffer)
        .expect("Failed to poll event")
    {
        use openxr::Event::*;
        match event {
            SessionStateChanged(state) => {
                use openxr::SessionState;

                let state = state.state();

                info!("entered XR state {:?}", state);

                let new_status = match state {
                    SessionState::IDLE => XrStatus::Idle,
                    SessionState::READY => XrStatus::Ready,
                    SessionState::SYNCHRONIZED | SessionState::VISIBLE | SessionState::FOCUSED => {
                        XrStatus::Running
                    }
                    SessionState::STOPPING => {
                        if app_exiting.0 {
                            if let Some(session) = &session {
                                session.end().expect("Failed to end session");
                            }

                            exit_app.send_default();
                        }

                        XrStatus::Stopping
                    }
                    SessionState::EXITING | SessionState::LOSS_PENDING => XrStatus::Exiting,
                    _ => unreachable!(),
                };

                status.set(new_status);
            }
            InstanceLossPending(_) => {}
            EventsLost(e) => warn!("lost {} XR events", e.lost_event_count()),
            _ => {}
        }
    }
}
