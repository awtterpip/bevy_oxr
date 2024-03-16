use bevy::app::{App, First, Plugin};
use bevy::ecs::event::{Event, EventWriter};
use bevy::ecs::schedule::common_conditions::resource_exists_and_changed;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{Res, Resource};
use bevy::render::extract_resource::ExtractResource;

pub struct XrSessionPlugin;

impl Plugin for XrSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateXrSession>()
            .add_event::<DestroyXrSession>()
            .add_event::<BeginXrSession>()
            .add_event::<EndXrSession>()
            .add_systems(
                First,
                handle_session.run_if(resource_exists_and_changed::<XrStatus>),
            );
    }
}

#[derive(Resource, ExtractResource, Clone, Copy, Debug, PartialEq, Eq)]
pub enum XrStatus {
    /// An XR session is not available here
    Unavailable,
    /// An XR session is available and ready to be created with a [`CreateXrSession`] event.
    Available,
    /// An XR session is created but not ready to begin.
    Idle,
    /// An XR session has been created and is ready to start rendering with a [`BeginXrSession`] event, or
    Ready,
    /// The XR session is running and can be stopped with an [`EndXrSession`] event.
    Running,
    /// The XR session is in the process of being stopped.
    Stopping,
    /// The XR session is in the process of being destroyed
    Exiting,
}

pub fn handle_session(
    status: Res<XrStatus>,
    mut create_session: EventWriter<CreateXrSession>,
    mut begin_session: EventWriter<BeginXrSession>,
    mut end_session: EventWriter<EndXrSession>,
) {
    match *status {
        XrStatus::Unavailable => {}
        XrStatus::Available => {
            create_session.send_default();
        }
        XrStatus::Idle => {}
        XrStatus::Ready => {
            begin_session.send_default();
        }
        XrStatus::Running => {}
        XrStatus::Stopping => {}
        XrStatus::Exiting => {}
    }
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is available. Returns true as long as [`XrStatus`] exists and isn't [`Unavailable`](XrStatus::Unavailable).
pub fn session_available(status: Option<Res<XrStatus>>) -> bool {
    status.is_some_and(|s| *s != XrStatus::Unavailable)
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is ready or running
pub fn session_created(status: Option<Res<XrStatus>>) -> bool {
    matches!(status.as_deref(), Some(XrStatus::Ready | XrStatus::Running))
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is running
pub fn session_running(status: Option<Res<XrStatus>>) -> bool {
    matches!(status.as_deref(), Some(XrStatus::Running))
}

/// A function that returns a [`Condition`](bevy::ecs::schedule::Condition) system that says if an the [`XrStatus`] is in a specific state
pub fn status_equals(status: XrStatus) -> impl FnMut(Option<Res<XrStatus>>) -> bool {
    move |state: Option<Res<XrStatus>>| state.is_some_and(|s| *s == status)
}

/// Event sent to backends to create an XR session. Should only be called in the [`XrStatus::Available`] state.
#[derive(Event, Clone, Copy, Default)]
pub struct CreateXrSession;

/// Event sent to the backends to destroy an XR session.
#[derive(Event, Clone, Copy, Default)]
pub struct DestroyXrSession;

/// Event sent to backends to begin an XR session. Should only be called in the [`XrStatus::Ready`] state.
#[derive(Event, Clone, Copy, Default)]
pub struct BeginXrSession;

/// Event sent to backends to end an XR session. Should only be called in the [`XrStatus::Running`] state.
#[derive(Event, Clone, Copy, Default)]
pub struct EndXrSession;
