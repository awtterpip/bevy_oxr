use crate::resources::OxrSession;
use bevy::prelude::*;
use bevy_xr::session::status_changed_to;
use openxr::ActionSet;

impl Plugin for OxrActionAttachingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OxrAttachActionSet>();
        app.add_systems(
            PostUpdate,
            attach_sets.run_if(status_changed_to(bevy_xr::session::XrStatus::Ready)),
        );
        app.init_resource::<AttachedActionSets>();
    }
}

fn attach_sets(
    session: Res<OxrSession>,
    mut events: EventReader<OxrAttachActionSet>,
    mut attached: ResMut<AttachedActionSets>,
) {
    let sets = events.read().map(|v| &v.0).collect::<Vec<_>>();
    if sets.is_empty() {
        return;
    }
    info!("attaching {} sessions", sets.len());
    match session.attach_action_sets(&sets) {
        Ok(_) => {
            info!("attached sessions!");
            for &set in sets.iter() {
                let clone = set.clone();
                attached.sets.push(clone);
            }
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

#[derive(Resource, Default)]
pub struct AttachedActionSets {
    pub sets: Vec<ActionSet>,
}
