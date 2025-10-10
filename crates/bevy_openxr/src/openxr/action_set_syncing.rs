use crate::{openxr_session_running, session::OxrSession};
use bevy::prelude::*;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct OxrActionSetSyncSet;

impl Plugin for OxrActionSyncingPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<OxrSyncActionSet>();
        app.add_systems(
            PreUpdate,
            sync_sets
                .in_set(OxrActionSetSyncSet)
                .run_if(openxr_session_running),
        );
    }
}

fn sync_sets(session: Res<OxrSession>, mut events: MessageReader<OxrSyncActionSet>) {
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

#[derive(Message, Clone)]
/// Send this event for every ActionSet you want to attach to the [`OxrSession`] once the Session Status changed to Ready. all requests will
pub struct OxrSyncActionSet(pub openxr::ActionSet);

pub struct OxrActionSyncingPlugin;
