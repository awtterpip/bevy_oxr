use std::sync::{Arc, RwLock};

use bevy::{ecs::schedule::ScheduleLabel, prelude::*, render::RenderApp};

pub struct XrSessionPlugin;

impl Plugin for XrSessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_schedule(XrSessionCreated);
        app.init_schedule(XrSessionExiting);
        app.add_event::<CreateXrSession>()
            .add_event::<DestroyXrSession>()
            .add_event::<BeginXrSession>()
            .add_event::<EndXrSession>()
            .add_event::<XrStatusChanged>()
            .add_systems(
                PreUpdate,
                handle_session.run_if(resource_exists::<XrSharedStatus>),
            );
    }
    fn finish(&self, app: &mut App) {
        // This is in finnish because we need the RenderPlugin to already be added.
        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_schedule(XrRenderSessionEnding);
    }
}

#[derive(ScheduleLabel, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct XrSessionCreated;

#[derive(ScheduleLabel, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct XrSessionExiting;

#[derive(ScheduleLabel, Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct XrRenderSessionEnding;

#[derive(Event, Clone, Copy, Deref)]
pub struct XrStatusChanged(pub XrStatus);

#[derive(Resource, Clone)]
pub struct XrSharedStatus(Arc<RwLock<XrStatus>>);

impl XrSharedStatus {
    pub fn new(status: XrStatus) -> Self {
        Self(Arc::new(RwLock::new(status)))
    }

    pub fn get(&self) -> XrStatus {
        *self.0.read().unwrap()
    }

    pub fn set(&self, status: XrStatus) {
        *self.0.write().unwrap() = status;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
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
    status: Res<XrSharedStatus>,
    mut previous_status: Local<Option<XrStatus>>,
    mut create_session: EventWriter<CreateXrSession>,
    mut begin_session: EventWriter<BeginXrSession>,
    mut end_session: EventWriter<EndXrSession>,
    mut destroy_session: EventWriter<DestroyXrSession>,
) {
    let current_status = status.get();
    if *previous_status != Some(current_status) {
        match current_status {
            XrStatus::Unavailable => {}
            XrStatus::Available => {
                create_session.send_default();
            }
            XrStatus::Idle => {}
            XrStatus::Ready => {
                begin_session.send_default();
            }
            XrStatus::Running => {}
            XrStatus::Stopping => {
                // end_session.send_default();
            }
            XrStatus::Exiting => {
                destroy_session.send_default();
            }
        }
    }
    *previous_status = Some(current_status);
}

/// A [`Condition`](bevy::ecs::schedule::Condition) that allows the system to run when the xr status changed to a specific [`XrStatus`].
pub fn status_changed_to(
    status: XrStatus,
) -> impl FnMut(EventReader<XrStatusChanged>) -> bool + Clone {
    move |mut reader: EventReader<XrStatusChanged>| {
        reader.read().any(|new_status| new_status.0 == status)
    }
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is available. Returns true as long as [`XrStatus`] exists and isn't [`Unavailable`](XrStatus::Unavailable).
pub fn session_available(status: Option<Res<XrSharedStatus>>) -> bool {
    status.is_some_and(|s| s.get() != XrStatus::Unavailable)
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is ready or running
pub fn session_created(status: Option<Res<XrSharedStatus>>) -> bool {
    matches!(
        status.as_deref().map(XrSharedStatus::get),
        Some(XrStatus::Ready | XrStatus::Running)
    )
}

/// A [`Condition`](bevy::ecs::schedule::Condition) system that says if the XR session is running
pub fn session_running(status: Option<Res<XrSharedStatus>>) -> bool {
    matches!(
        status.as_deref().map(XrSharedStatus::get),
        Some(XrStatus::Running)
    )
}

/// A function that returns a [`Condition`](bevy::ecs::schedule::Condition) system that says if an the [`XrStatus`] is in a specific state
pub fn status_equals(status: XrStatus) -> impl FnMut(Option<Res<XrSharedStatus>>) -> bool {
    move |state: Option<Res<XrSharedStatus>>| state.is_some_and(|s| s.get() == status)
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
