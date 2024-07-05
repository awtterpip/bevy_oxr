use bevy::app::{AppExit, MainScheduleOrder};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::{Render, RenderApp, RenderSet};

/// Event sent to instruct backends to create an XR session. Only works when the [`XrState`] is [`Available`](XrState::Available).
#[derive(Event, Clone, Copy, Default)]
pub struct XrCreateSessionEvent;

/// A schedule thats ran whenever an [`XrCreateSessionEvent`] is recieved while the [`XrState`] is [`Available`](XrState::Available)
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, ScheduleLabel)]
pub struct XrCreateSession;

/// Event sent when [`XrCreateSession`] is ran
#[derive(Event, Clone, Copy, Default)]
pub struct XrSessionCreatedEvent;

/// Event sent to instruct backends to destroy an XR session. Only works when the [`XrState`] is [`Exiting`](XrState::Exiting).
/// If you would like to request that a running session be destroyed, send the [`XrRequestExitEvent`] instead.
#[derive(Event, Clone, Copy, Default)]
pub struct XrDestroySessionEvent;

/// Resource flag thats inserted into the world and extracted to the render world to inform any session resources in the render world to drop.
#[derive(Resource, ExtractResource, Clone, Copy, Default)]
pub struct XrDestroySessionRender;

/// Schedule thats ran whenever an [`XrDestroySessionEvent`] is recieved while the [`XrState`] is [`Exiting`](XrState::Exiting).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, ScheduleLabel)]
pub struct XrDestroySession;

/// Event sent to instruct backends to begin an XR session. Only works when the [`XrState`] is [`Ready`](XrState::Ready).
#[derive(Event, Clone, Copy, Default)]
pub struct XrBeginSessionEvent;

/// Schedule thats ran whenever an [`XrBeginSessionEvent`] is recieved while the [`XrState`] is [`Ready`](XrState::Ready).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, ScheduleLabel)]
pub struct XrBeginSession;

/// Event sent to backends to end an XR session. Only works when the [`XrState`] is [`Stopping`](XrState::Stopping).
#[derive(Event, Clone, Copy, Default)]
pub struct XrEndSessionEvent;

/// Schedule thats rna whenever an [`XrEndSessionEvent`] is recieved while the [`XrState`] is [`Stopping`](XrState::Stopping).
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, ScheduleLabel)]
pub struct XrEndSession;

/// Event sent to backends to request the [`XrState`] proceed to [`Exiting`](XrState::Exiting) and for the session to be exited. Can be called at any time a session exists.
#[derive(Event, Clone, Copy, Default)]
pub struct XrRequestExitEvent;

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, ScheduleLabel)]
pub struct XrRequestExit;

/// Schedule ran before [`First`] to handle XR events.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, ScheduleLabel)]
pub struct XrFirst;

/// System set for systems related to handling XR session events and updating the [`XrState`]
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub struct XrHandleEvents;

/// System sets ran in the render world for XR.
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, SystemSet)]
pub enum XrRenderSet {
    /// Ran before [`XrRenderSet::PreRender`] but after [`RenderSet::ExtractCommands`].
    HandleEvents,
    /// For any XR systems needing to be ran before rendering begins.
    /// Ran after [`XrRenderSet::HandleEvents`] but before every render set except [`RenderSet::ExtractCommands`].
    PreRender,
    /// For any XR systems needing to be ran after [`RenderSet::Render`] but before [`RenderSet::Cleanup`].
    PostRender,
}

/// The root transform's global position for late latching in the render world.
#[derive(ExtractResource, Resource, Clone, Copy, Default)]
pub struct XrRootTransform(pub GlobalTransform);

/// Component used to specify the entity we should use as the tracking root.
#[derive(Component)]
pub struct XrTrackingRoot;

pub struct XrSessionPlugin {
    pub auto_handle: bool,
}

impl Plugin for XrSessionPlugin {
    fn build(&self, app: &mut App) {
        let mut xr_first = Schedule::new(XrFirst);
        xr_first.set_executor_kind(bevy::ecs::schedule::ExecutorKind::Simple);
        app.add_event::<XrCreateSessionEvent>()
            .add_event::<XrDestroySessionEvent>()
            .add_event::<XrBeginSessionEvent>()
            .add_event::<XrEndSessionEvent>()
            .add_event::<XrRequestExitEvent>()
            .add_event::<XrStateChanged>()
            .add_event::<XrSessionCreatedEvent>()
            .init_schedule(XrCreateSession)
            .init_schedule(XrDestroySession)
            .init_schedule(XrBeginSession)
            .init_schedule(XrEndSession)
            .init_schedule(XrRequestExit)
            .add_schedule(xr_first)
            .add_systems(
                XrFirst,
                (
                    exits_session_on_app_exit
                        .run_if(on_event::<AppExit>())
                        .run_if(session_created),
                    reset_per_frame_resources,
                    run_xr_create_session
                        .run_if(state_equals(XrState::Available))
                        .run_if(on_event::<XrCreateSessionEvent>()),
                    run_xr_destroy_session
                        .run_if(state_matches!(XrState::Exiting { .. }))
                        .run_if(on_event::<XrDestroySessionEvent>()),
                    run_xr_begin_session
                        .run_if(state_equals(XrState::Ready))
                        .run_if(on_event::<XrBeginSessionEvent>()),
                    run_xr_end_session
                        .run_if(state_equals(XrState::Stopping))
                        .run_if(on_event::<XrEndSessionEvent>()),
                    run_xr_request_exit
                        .run_if(session_created)
                        .run_if(on_event::<XrRequestExitEvent>()),
                )
                    .chain()
                    .in_set(XrHandleEvents),
            );

        app.world
            .resource_mut::<MainScheduleOrder>()
            .labels
            .insert(0, XrFirst.intern());

        if self.auto_handle {
            app.add_systems(PreUpdate, auto_handle_session);
        }
    }

    fn finish(&self, app: &mut App) {
        if app.get_sub_app(RenderApp).is_err() {
            return;
        }

        app.add_plugins((
            ExtractResourcePlugin::<XrState>::default(),
            ExtractResourcePlugin::<XrDestroySessionRender>::default(),
            ExtractResourcePlugin::<XrRootTransform>::default(),
        ))
        .init_resource::<XrRootTransform>()
        .add_systems(
            PostUpdate,
            update_root_transform.after(TransformSystem::TransformPropagate),
        )
        .add_systems(
            XrFirst,
            exits_session_on_app_exit
                .before(XrHandleEvents)
                .run_if(on_event::<AppExit>().and_then(session_running)),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_schedule(XrDestroySession)
            .init_resource::<XrRootTransform>()
            .configure_sets(
                Render,
                (XrRenderSet::HandleEvents, XrRenderSet::PreRender).chain(),
            )
            .configure_sets(
                Render,
                XrRenderSet::HandleEvents.after(RenderSet::ExtractCommands),
            )
            .configure_sets(
                Render,
                XrRenderSet::PreRender
                    .before(RenderSet::ManageViews)
                    .before(RenderSet::PrepareAssets),
            )
            .configure_sets(
                Render,
                XrRenderSet::PostRender
                    .after(RenderSet::Render)
                    .before(RenderSet::Cleanup),
            )
            .add_systems(
                Render,
                (
                    run_xr_destroy_session
                        .run_if(resource_exists::<XrDestroySessionRender>)
                        .in_set(XrRenderSet::HandleEvents),
                    reset_per_frame_resources.in_set(RenderSet::Cleanup),
                ),
            );
    }
}

fn exits_session_on_app_exit(mut request_exit: EventWriter<XrRequestExitEvent>) {
    request_exit.send_default();
}

/// Event sent by backends whenever [`XrState`] is changed.
#[derive(Event, Clone, Copy, Deref)]
pub struct XrStateChanged(pub XrState);

/// A resource in the main world and render world representing the current session state.
#[derive(Clone, Copy, Debug, ExtractResource, Resource, PartialEq, Eq)]
#[repr(u8)]
pub enum XrState {
    /// An XR session is not available here
    Unavailable,
    /// An XR session is available and ready to be created with an [`XrCreateSessionEvent`].
    Available,
    /// An XR session is created but not ready to begin. Backends are not required to use this state.
    Idle,
    /// An XR session has been created and is ready to start rendering with an [`XrBeginSessionEvent`].
    Ready,
    /// The XR session is running and can be stopped with an [`XrEndSessionEvent`].
    Running,
    /// The runtime has requested that the session should be ended with an [`XrEndSessionEvent`].
    Stopping,
    /// The XR session should be destroyed with an [`XrDestroySessionEvent`].
    Exiting {
        /// Whether we should automatically restart the session
        should_restart: bool,
    },
}

pub fn run_xr_create_session(world: &mut World) {
    world.run_schedule(XrCreateSession);
    world.send_event(XrSessionCreatedEvent);
}

pub fn run_xr_destroy_session(world: &mut World) {
    world.run_schedule(XrDestroySession);
    world.insert_resource(XrDestroySessionRender);
}

pub fn run_xr_begin_session(world: &mut World) {
    world.run_schedule(XrBeginSession);
}

pub fn run_xr_end_session(world: &mut World) {
    world.run_schedule(XrEndSession);
}

pub fn run_xr_request_exit(world: &mut World) {
    world.run_schedule(XrRequestExit);
}

pub fn reset_per_frame_resources(world: &mut World) {
    world.remove_resource::<XrDestroySessionRender>();
}

pub fn auto_handle_session(
    mut state_changed: EventReader<XrStateChanged>,
    mut create_session: EventWriter<XrCreateSessionEvent>,
    mut begin_session: EventWriter<XrBeginSessionEvent>,
    mut end_session: EventWriter<XrEndSessionEvent>,
    mut destroy_session: EventWriter<XrDestroySessionEvent>,
) {
    for XrStateChanged(state) in state_changed.read() {
        match state {
            XrState::Available => {
                create_session.send_default();
            }
            XrState::Ready => {
                begin_session.send_default();
            }
            XrState::Stopping => {
                end_session.send_default();
            }
            XrState::Exiting { .. } => {
                destroy_session.send_default();
            }
            _ => (),
        }
    }
}

pub fn update_root_transform(
    mut root_transform: ResMut<XrRootTransform>,
    root: Query<&GlobalTransform, With<XrTrackingRoot>>,
) {
    let Ok(transform) = root.get_single() else {
        return;
    };

    root_transform.0 = *transform;
}

/// A [`Condition`](bevy::ecs::schedule::Condition) that allows the system to run when the xr status changed to a specific [`XrStatus`].
pub fn status_changed_to(
    status: XrState,
) -> impl FnMut(EventReader<XrStateChanged>) -> bool + Clone {
    move |mut reader: EventReader<XrStateChanged>| {
        reader.read().any(|new_status| new_status.0 == status)
    }
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is available. Returns true as long as [`XrState`] exists and isn't [`Unavailable`](XrStatus::Unavailable).
pub fn session_available(status: Option<Res<XrState>>) -> bool {
    status.is_some_and(|s| *s != XrState::Unavailable)
}

pub fn session_created(status: Option<Res<XrState>>) -> bool {
    !matches!(
        status.as_deref(),
        Some(XrState::Unavailable | XrState::Available) | None
    )
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is ready or running
pub fn session_ready_or_running(status: Option<Res<XrState>>) -> bool {
    matches!(status.as_deref(), Some(XrState::Ready | XrState::Running))
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is running
pub fn session_running(status: Option<Res<XrState>>) -> bool {
    matches!(status.as_deref(), Some(XrState::Running))
}

/// A function that returns a [`Condition`](bevy::ecs::schedule::Condition) system that says if the [`XrState`] is in a specific state
pub fn state_equals(status: XrState) -> impl FnMut(Option<Res<XrState>>) -> bool {
    move |state: Option<Res<XrState>>| state.is_some_and(|s| *s == status)
}

#[macro_export]
macro_rules! state_matches {
    ($match:pat) => {
        (|state: Option<Res<XrState>>| core::matches!(state.as_deref(), Some($match)))
    };
}

use bevy::transform::TransformSystem;
pub use state_matches;
