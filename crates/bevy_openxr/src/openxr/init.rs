use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::Mutex;

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResourcePlugin;
use bevy::render::renderer::RenderAdapter;
use bevy::render::renderer::RenderAdapterInfo;
use bevy::render::renderer::RenderInstance;
use bevy::render::renderer::RenderQueue;
use bevy::render::settings::RenderCreation;
use bevy::render::Render;
use bevy::render::RenderApp;
use bevy::render::RenderPlugin;
use bevy::render::RenderSet;
use bevy::transform::TransformSystem;
use bevy::utils::synccell::SyncCell;
use bevy::winit::UpdateMode;
use bevy::winit::WinitSettings;
use bevy_xr::session::session_available;
use bevy_xr::session::status_equals;
use bevy_xr::session::BeginXrSession;
use bevy_xr::session::CreateXrSession;
use bevy_xr::session::DestroyXrSession;
use bevy_xr::session::EndXrSession;
use bevy_xr::session::XrSharedStatus;
use bevy_xr::session::XrStatus;
use bevy_xr::session::XrStatusChanged;

use crate::error::OxrError;
use crate::graphics::*;
use crate::resources::*;
use crate::types::*;

pub fn session_started(started: Option<Res<OxrSessionStarted>>) -> bool {
    started.is_some_and(|started| started.get())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum OxrPreUpdateSet {
    WaitFrame,
    UpdateCriticalComponents,
    UpdateNonCriticalComponents,
    SyncActions,
}

/// This system set runs in the [Render] schedule before [RenderSet::ExtractCommands]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub struct OxrPollMain;

pub struct PrePoll;

/// Plugin that handles the initialization of OpenXR resources
pub struct OxrInitPlugin {
    /// Information about the app this is being used to build.
    pub app_info: AppInfo,
    /// Extensions wanted for this session.
    // TODO!() This should be changed to take a simpler list of features wanted that this crate supports. i.e. hand tracking
    pub exts: OxrExtensions,
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

impl Default for OxrInitPlugin {
    fn default() -> Self {
        Self {
            app_info: default(),
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_fb_passthrough();
                exts.enable_hand_tracking();
                exts
            },
            blend_modes: default(),
            backends: default(),
            formats: Some(vec![wgpu::TextureFormat::Rgba8UnormSrgb]),
            resolutions: default(),
            synchronous_pipeline_compilation: default(),
        }
    }
}

#[derive(Component)]
pub struct OxrTrackingRoot;

#[derive(Resource)]
pub struct PollSender(SyncSender<()>);

#[derive(Resource)]
pub struct PollReciever(SyncCell<Receiver<()>>);

pub trait OxrSessionResourceCreator {
    /// Initialize the resource and store it here
    fn initialize(&mut self, world: &mut World) -> Result<()>;
    /// Insert any resources into the main world here. Ran after `initialize`.
    fn insert_to_world(&mut self, world: &mut World);
    /// Insert any resources into the render world here. Ran after `insert_to_world`
    fn insert_to_render_world(&mut self, world: &mut World);
    /// This methods is automatically ran whenever [destroy_xr_session_resources] is ran, triggered by a [DestroyXrSession] event.
    fn remove_from_world(&mut self, world: &mut World);
    /// This method is automatically ran whenever [destroy_xr_session_resources] is ran, triggered by a [DestroyXrSession] event.
    fn remove_from_render_world(&mut self, world: &mut World);
}

#[derive(Clone, Default, Resource)]
pub struct OxrSessionResourceCreators(
    Arc<Mutex<(Vec<(Box<dyn OxrSessionResourceCreator + Send>, bool)>, bool)>>,
);

pub trait OxrSessionResourceCreatorsApp {
    fn add_xr_resource_creator<R: OxrSessionResourceCreator + Send + 'static>(&self, resource: R);

    fn init_xr_resource_creator<R: OxrSessionResourceCreator + Send + Default + 'static>(&self) {
        self.add_xr_resource_creator(R::default())
    }
}

impl OxrSessionResourceCreatorsApp for OxrSessionResourceCreators {
    fn add_xr_resource_creator<R: OxrSessionResourceCreator + Send + 'static>(&self, resource: R) {
        let mut resources = self.0.lock().unwrap();
        resources.0.push((Box::new(resource), false));
    }
}

impl OxrSessionResourceCreatorsApp for App {
    fn add_xr_resource_creator<R: OxrSessionResourceCreator + Send + 'static>(&self, resource: R) {
        self.world
            .resource::<OxrSessionResourceCreators>()
            .add_xr_resource_creator(resource);
    }
}

impl Plugin for OxrInitPlugin {
    fn build(&self, app: &mut App) {
        match self.init_xr() {
            Ok((
                instance,
                system_id,
                WgpuGraphics(device, queue, adapter_info, adapter, wgpu_instance),
                session_create_info,
            )) => {
                let status = XrSharedStatus::new(XrStatus::Available);
                let (tx, rx) = sync_channel::<()>(0);
                let render_resources = OxrSessionResourceCreators::default();
                render_resources.init_xr_resource_creator::<OxrSessionResources>();
                let cleanup_session = OxrCleanupSession::default();

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
                    ExtractResourcePlugin::<OxrRootTransform>::default(),
                ))
                .add_systems(
                    First,
                    (
                        reset_per_frame_resources,
                        poll_events.run_if(session_available),
                        create_xr_session_resources
                            .run_if(on_event::<CreateXrSession>())
                            .run_if(status_equals(XrStatus::Available)),
                        begin_xr_session
                            .run_if(on_event::<BeginXrSession>())
                            .run_if(status_equals(XrStatus::Ready)),
                        end_xr_session
                            .run_if(on_event::<EndXrSession>())
                            .run_if(status_equals(XrStatus::Stopping)),
                        destroy_xr_session_resources
                            .run_if(on_event::<DestroyXrSession>())
                            .run_if(status_equals(XrStatus::Exiting)),
                        finish_poll,
                    )
                        .chain()
                        .run_if(not(resource_added::<XrSharedStatus>)),
                )
                .add_systems(
                    PostUpdate,
                    update_root_transform.after(TransformSystem::TransformPropagate),
                )
                .insert_resource(PollSender(tx))
                .insert_resource(render_resources.clone())
                .insert_resource(cleanup_session.clone())
                .insert_resource(instance.clone())
                .insert_resource(system_id)
                .insert_resource(status.clone())
                .insert_resource(WinitSettings {
                    focused_mode: UpdateMode::Continuous,
                    unfocused_mode: UpdateMode::Continuous,
                })
                .init_resource::<OxrRootTransform>()
                .insert_non_send_resource(session_create_info);

                app.world
                    .spawn((TransformBundle::default(), OxrTrackingRoot));

                let render_app = app.sub_app_mut(RenderApp);
                render_app
                    .insert_resource(PollReciever(SyncCell::new(rx)))
                    .insert_resource(render_resources.clone())
                    .insert_resource(instance)
                    .insert_resource(system_id)
                    .insert_resource(status)
                    .insert_resource(cleanup_session)
                    .init_resource::<OxrRootTransform>()
                    .init_resource::<OxrCleanupSession>()
                    .init_resource::<OxrRenderLayers>()
                    .add_systems(Render, handle_xr_events_render.in_set(OxrPollMain));
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
                OxrPreUpdateSet::WaitFrame,
                OxrPreUpdateSet::UpdateCriticalComponents,
                OxrPreUpdateSet::UpdateNonCriticalComponents,
            )
                .chain(),
        );

        let session_started = OxrSessionStarted::default();

        app.insert_resource(session_started.clone());

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            // .configure_sets(
            //     Render,
            //     OxrPollMain
            //         .after(RenderSet::ExtractCommands)
            //         .before(RenderSet::PrepareAssets)
            //         .before(RenderSet::ManageViews),
            // )
            .configure_sets(Render, OxrPollMain.before(RenderSet::ExtractCommands))
            .insert_resource(session_started);
    }
}

fn xr_entry() -> Result<OxrEntry> {
    #[cfg(windows)]
    let entry = openxr::Entry::linked();
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(OxrEntry(entry))
}

impl OxrInitPlugin {
    fn init_xr(&self) -> Result<(OxrInstance, OxrSystemId, WgpuGraphics, OxrSessionConfigInfo)> {
        let entry = xr_entry()?;

        #[cfg(target_os = "android")]
        entry.initialize_android_loader()?;

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
        .ok_or(OxrError::NoAvailableBackend)?;

        let exts = self.exts.clone() & available_exts;

        let instance = entry.create_instance(
            self.app_info.clone(),
            exts,
            // &["XR_APILAYER_LUNARG_api_dump"],
            &[],
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

        let session_create_info = OxrSessionConfigInfo {
            blend_modes: self.blend_modes.clone(),
            formats: self.formats.clone(),
            resolutions: self.resolutions.clone(),
            graphics_info,
        };

        Ok((
            instance,
            OxrSystemId(system_id),
            graphics,
            session_create_info,
        ))
    }
}

#[derive(Resource, Default)]
struct OxrSessionResources {
    session: Option<OxrSession>,
    frame_stream: Option<OxrFrameStream>,
    frame_waiter: Option<OxrFrameWaiter>,
}

impl OxrSessionResourceCreator for OxrSessionResources {
    /// This is where we create the session and initialize the session resources
    fn initialize(&mut self, world: &mut World) -> Result<()> {
        let session_config_info = world.non_send_resource::<OxrSessionConfigInfo>();
        let system_id = world.resource::<OxrSystemId>();
        let instance = world.resource::<OxrInstance>();

        let (session, frame_waiter, frame_stream) = unsafe {
            instance.create_session(**system_id, session_config_info.graphics_info.clone())?
        };
        self.session = Some(session);
        self.frame_waiter = Some(frame_waiter);
        self.frame_stream = Some(frame_stream);
        Ok(())
    }

    /// We can selectively choose to add frame waiter to the main world only
    fn insert_to_world(&mut self, world: &mut World) {
        world.insert_resource(self.session.clone().unwrap());
        world.insert_resource(self.frame_waiter.take().unwrap());
    }

    /// And add frame stream to the render world only
    fn insert_to_render_world(&mut self, world: &mut World) {
        world.insert_resource(self.session.take().unwrap());
        world.insert_resource(self.frame_stream.take().unwrap());
    }

    fn remove_from_world(&mut self, world: &mut World) {
        world.remove_resource::<OxrSession>();
        world.remove_resource::<OxrFrameWaiter>();
    }

    fn remove_from_render_world(&mut self, world: &mut World) {
        world.remove_resource::<OxrSession>();
        world.remove_resource::<OxrFrameStream>();
        world.resource_mut::<OxrRenderLayers>().clear();
    }
}

pub fn create_xr_session_resources(world: &mut World) {
    let resource_creators = world.resource::<OxrSessionResourceCreators>().clone();
    let mut resource_creators_mut = resource_creators.0.lock().unwrap();

    for (resource_creator, initialization_succeeded) in &mut resource_creators_mut.0 {
        match resource_creator.initialize(world) {
            Ok(_) => {
                *initialization_succeeded = true;
                resource_creator.insert_to_world(world);
            }
            Err(e) => error!(
                "Failed to initialize a resource using resource creator '{}': {}",
                std::any::type_name_of_val(resource_creator),
                e
            ),
        }
    }
    // This signifies to the render world that a new session has been created and that it should attempt to insert any render resources
    resource_creators_mut.1 = true;
}

pub fn begin_xr_session(session: Res<OxrSession>, session_started: Res<OxrSessionStarted>) {
    let _span = info_span!("xr_begin_session");
    session
        .begin(openxr::ViewConfigurationType::PRIMARY_STEREO)
        .expect("Failed to begin session");
    session_started.set(true);
}

pub fn end_xr_session(session: Res<OxrSession>, session_started: Res<OxrSessionStarted>) {
    let _span = info_span!("xr_end_session");
    session.end().expect("Failed to end session");
    session_started.set(false);
}

pub fn handle_xr_events_render(world: &mut World) {
    let _span = info_span!("xr_handle_events_render");
    world.resource_mut::<PollReciever>().0.get().recv().unwrap();

    let resource_creators = world.resource::<OxrSessionResourceCreators>().clone();
    let mut resource_creators_mut = resource_creators.0.lock().unwrap();
    if resource_creators_mut.1 {
        for (resource_creator, initialized) in &mut resource_creators_mut.0 {
            if *initialized {
                resource_creator.insert_to_render_world(world);
            }
        }
        resource_creators_mut.1 = false;
    }
    if world.resource::<OxrCleanupSession>().get() {
        for (resource_creator, _) in &mut resource_creators_mut.0 {
            resource_creator.remove_from_render_world(world);
        }
    }
    if let Some(frame_state) = *world.resource::<OxrFrameState>().clone().0.lock().unwrap() {
        world.insert_resource(OxrTime(frame_state.predicted_display_time));
    }
}

pub fn finish_poll(sender: Res<PollSender>) {
    let _span = info_span!("xr_finish_poll");
    sender.0.send(()).unwrap();
}

/// Polls any OpenXR events and handles them accordingly
pub fn poll_events(
    instance: Res<OxrInstance>,
    status: Res<XrSharedStatus>,
    mut changed_event: EventWriter<XrStatusChanged>,
) {
    let _span = info_span!("xr_poll_events");
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
                    SessionState::STOPPING => XrStatus::Stopping,
                    SessionState::EXITING | SessionState::LOSS_PENDING => XrStatus::Exiting,
                    _ => unreachable!(),
                };
                changed_event.send(XrStatusChanged(new_status));
                status.set(new_status);
            }
            InstanceLossPending(_) => {}
            EventsLost(e) => warn!("lost {} XR events", e.lost_event_count()),
            _ => {}
        }
    }
}

pub fn reset_per_frame_resources(cleanup: ResMut<OxrCleanupSession>) {
    cleanup.set(false);
}

pub fn destroy_xr_session_resources(world: &mut World) {
    let resource_creators = world.resource::<OxrSessionResourceCreators>().clone();
    let mut resource_creators_mut = resource_creators.0.lock().unwrap();

    for (resource_creator, initialized) in &mut resource_creators_mut.0 {
        *initialized = false;
        resource_creator.remove_from_world(world);
    }
    world.resource::<OxrCleanupSession>().set(true);
}

pub fn update_root_transform(
    mut root_transform: ResMut<OxrRootTransform>,
    root: Query<&GlobalTransform, With<OxrTrackingRoot>>,
) {
    let transform = root.single();

    root_transform.0 = *transform;
}
