use crate::session::OxrSession;
use bevy::prelude::*;
use bevy_xr::session::session_running;

/// This is run in PreUpdate
#[derive(SystemSet, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct OxrActionSetSyncSet;
impl Plugin for OxrActionSyncingPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OxrSyncActionSet>();
        app.add_systems(
            PreUpdate,
            sync_sets
                .run_if(session_running)
                .in_set(OxrActionSetSyncSet), // .in_set(OxrPreUpdateSet::SyncActions),
        );
    }
}

fn sync_sets(session: Res<OxrSession>, mut events: EventReader<OxrSyncActionSet>) {
    let sets = events
        .read()
        .map(|v| &v.0)
        .map(openxr::ActiveActionSet::new)
        .collect::<Vec<_>>();

    if sets.is_empty() {
        return;
    }

    if let Err(err) = session.sync_actions(&sets) {
        warn!("error while syncing actionsets: {}", err.to_string());
    }
}

#[derive(Event, Clone)]
/// Send this event for every ActionSet you want to attach to the [`OxrSession`] once the Session Status changed to Ready. all requests will
pub struct OxrSyncActionSet(pub openxr::ActionSet);

pub struct OxrActionSyncingPlugin;
