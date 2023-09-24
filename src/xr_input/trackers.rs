use bevy::prelude::{Added, BuildChildren, Commands, Entity, Query, With, Res, Transform, Without, Component, info};

use crate::{resources::{XrFrameState, XrInstance, XrSession}, input::XrInput};

use super::{oculus_touch::OculusController, Hand, Vec3Conv, QuatConv};

#[derive(Component)]
pub struct XrTrackingRoot;
#[derive(Component)]
pub struct XrTracker;
#[derive(Component)]
pub struct XrLeftEye;
#[derive(Component)]
pub struct XrRightEye;
#[derive(Component)]
pub struct XrHmd;
#[derive(Component)]
pub struct XrLeftController;
#[derive(Component)]
pub struct XrRightController;
#[derive(Component)]
pub struct XrController;

pub fn adopt_open_xr_trackers(
    query: Query<(Entity), Added<XrTracker>>,
    mut commands: Commands,
    tracking_root_query: Query<(Entity, With<XrTrackingRoot>)>,
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

pub fn update_open_xr_controllers(
    oculus_controller: Res<OculusController>,
    mut left_controller_query: Query<(
        &mut Transform,
        With<XrLeftController>,
        Without<XrRightController>,
    )>,
    mut right_controller_query: Query<(
        &mut Transform,
        With<XrRightController>,
        Without<XrLeftController>,
    )>,
    frame_state: Res<XrFrameState>,
    instance: Res<XrInstance>,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
) {
    //lock dat frame?
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);
    //get left controller
    let (left, _) = controller.grip_space(Hand::Left);

    if let Ok(mut left_controller) = left_controller_query.get_single_mut() {
        left_controller.0.translation = left.pose.position.to_vec3();
        left_controller.0.rotation = left.pose.orientation.to_quat();
    }

    //get right controller
    let (right, _) = controller.grip_space(Hand::Right);

    if let Ok(mut right_controller) = right_controller_query.get_single_mut() {
        right_controller.0.translation = right.pose.position.to_vec3();
        right_controller.0.rotation = right.pose.orientation.to_quat()
    }
}

