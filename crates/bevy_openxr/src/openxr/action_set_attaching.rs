use crate::resources::OxrSession;
use bevy::prelude::*;
use bevy_xr::session::status_changed_to;

impl Plugin for OxrActionAttachingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OxrAttachActionSet>();
        app.add_systems(
            PostUpdate,
            attach_sets.run_if(status_changed_to(bevy_xr::session::XrStatus::Ready)),
        );
    }
}

fn attach_sets(session: Res<OxrSession>, mut events: EventReader<OxrAttachActionSet>) {
    let sets = events.read().map(|v| &v.0).collect::<Vec<_>>();
    match session.attach_action_sets(&sets) {
        Ok(_) => {}
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
