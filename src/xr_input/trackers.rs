use bevy::prelude::{info, Added, BuildChildren, Commands, Component, Entity, Query, With};

#[derive(Component)]
pub struct OpenXRTrackingRoot;
#[derive(Component)]
pub struct OpenXRTracker;
#[derive(Component)]
pub struct OpenXRLeftEye;
#[derive(Component)]
pub struct OpenXRRightEye;
#[derive(Component)]
pub struct OpenXRHMD;
#[derive(Component)]
pub struct OpenXRLeftController;
#[derive(Component)]
pub struct OpenXRRightController;
#[derive(Component)]
pub struct OpenXRController;

pub fn adopt_open_xr_trackers(
    query: Query<(Entity), Added<OpenXRTracker>>,
    mut commands: Commands,
    tracking_root_query: Query<(Entity, With<OpenXRTrackingRoot>)>,
) {
    let root = tracking_root_query.get_single();
    match root {
        Ok(thing) => {
            // info!("root is");
            for tracker in query.iter() {
                info!("we got a new tracker");
                commands.entity(thing.0).add_child(tracker);
            }
        }
        Err(_) => info!("root isnt spawned yet?"),
    }
}
