use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::event::{Event, EventReader, EventWriter};
use bevy::ecs::system::Local;

pub struct XrSessionPlugin;

impl Plugin for XrSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CreateXrSession>()
            .add_event::<BeginXrSession>()
            .add_event::<EndXrSession>()
            .add_event::<XrSessionState>()
            .add_event::<XrInstanceCreated>()
            .add_event::<XrInstanceDestroyed>()
            .add_systems(PreUpdate, handle_xr_events);
    }
}

pub fn handle_xr_events(
    mut instance_created: EventReader<XrInstanceCreated>,
    mut session_state: EventReader<XrSessionState>,
    mut instance_destroyed: EventReader<XrInstanceDestroyed>,
    mut create_session: EventWriter<CreateXrSession>,
    mut begin_session: EventWriter<BeginXrSession>,
    mut has_instance: Local<bool>,
    mut local_session_state: Local<Option<XrSessionState>>,
) {
    // Don't do anything if no events recieved
    if instance_created.is_empty() && instance_destroyed.is_empty() && session_state.is_empty() {
        return;
    }
    if !instance_created.is_empty() {
        *has_instance = true;
        instance_created.clear();
    }
    if !instance_destroyed.is_empty() {
        *has_instance = false;
        instance_destroyed.clear();
    }
    for state in session_state.read() {
        *local_session_state = Some(*state);
    }
    if *has_instance {
        if local_session_state.is_none() {
            create_session.send_default();
        } else if matches!(*local_session_state, Some(XrSessionState::Ready)) {
            begin_session.send_default();
        }
    }
}

/// Event sent to backends to create an XR session
#[derive(Event, Clone, Copy, Default)]
pub struct CreateXrSession;

/// Event sent to backends to begin an XR session
#[derive(Event, Clone, Copy, Default)]
pub struct BeginXrSession;

/// Event sent to backends to end an XR session.
#[derive(Event, Clone, Copy, Default)]
pub struct EndXrSession;

// /// Event sent to backends to destroy an XR session.
// #[derive(Event, Clone, Copy, Default)]
// pub struct DestroyXrSession;

/// Event sent from backends to inform the frontend of the session state.
#[derive(Event, Clone, Copy)]
pub enum XrSessionState {
    /// The session is in an idle state. Either just created or stopped
    Idle,
    /// The session is ready. You may send a [`BeginXrSession`] event.
    Ready,
    /// The session is running.
    Running,
    /// The session is being stopped
    Stopping,
    /// The session is destroyed
    Destroyed,
}

/// Event sent from backends to inform the frontend that the instance was created.
#[derive(Event, Clone, Copy, Default)]
pub struct XrInstanceCreated;

/// Event sent from backends to inform the frontend that the instance was destroyed.
#[derive(Event, Clone, Copy, Default)]
pub struct XrInstanceDestroyed;
