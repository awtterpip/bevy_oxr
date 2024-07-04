use crate::{action_binding::run_action_binding_sugestion, session::OxrSession};
use bevy::prelude::*;
use bevy_mod_xr::session::XrSessionCreatedEvent;

impl Plugin for OxrActionAttachingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OxrAttachActionSet>();
        app.add_systems(
            PostUpdate,
            attach_sets
                .run_if(on_event::<XrSessionCreatedEvent>())
                .after(run_action_binding_sugestion),
        );
    }
}

fn attach_sets(session: Res<OxrSession>, mut events: EventReader<OxrAttachActionSet>) {
    let sets = events.read().map(|v| &v.0).collect::<Vec<_>>();
    if sets.is_empty() {
        return;
    }
    info!("attaching {} sessions", sets.len());
    match session.attach_action_sets(&sets) {
        Ok(_) => {
            info!("attached sessions!")
        }
        Err(openxr::sys::Result::ERROR_ACTIONSETS_ALREADY_ATTACHED) => {
            error!("Action Sets Already attached!");
        }

        Err(openxr::sys::Result::ERROR_HANDLE_INVALID) => error!("Invalid ActionSet Handle!"),
        Err(e) => error!(
            "Unhandled Error while attaching action sets: {}",
            e.to_string()
        ),
    };
}

#[derive(Event, Clone)]
/// Send this event for every ActionSet you want to attach to the [`OxrSession`] once the Session Status changed to Ready. all requests will
/// be applied in [`PostUpdate`]
pub struct OxrAttachActionSet(pub openxr::ActionSet);

pub struct OxrActionAttachingPlugin;
