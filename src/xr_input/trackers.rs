use bevy::hierarchy::Parent;
use bevy::log::{debug, info};
use bevy::prelude::{
    Added, BuildChildren, Commands, Component, Entity, Query, Res, Transform, Vec3, With, Without,
};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
};

use super::{actions::XrActionSets, oculus_touch::OculusController, Hand, QuatConv, Vec3Conv};

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
#[derive(Component)]
pub struct AimPose(pub Transform);

pub fn adopt_open_xr_trackers(
    query: Query<Entity, (With<OpenXRTracker>, Without<Parent>)>,
    mut commands: Commands,
    tracking_root_query: Query<Entity, With<OpenXRTrackingRoot>>,
) {
    let root = tracking_root_query.get_single();
    match root {
        Ok(root) => {
            // info!("root is");
            for tracker in query.iter() {
                info!("we got a new tracker");
                commands.entity(root).add_child(tracker);
            }
        }
        Err(_) => info!("root isnt spawned yet?"),
    }
}

pub fn update_open_xr_controllers(
    oculus_controller: Res<OculusController>,
    mut left_controller_query: Query<(
        &mut Transform,
        Option<&mut AimPose>,
        With<OpenXRLeftController>,
        Without<OpenXRRightController>,
    )>,
    mut right_controller_query: Query<(
        &mut Transform,
        Option<&mut AimPose>,
        With<OpenXRRightController>,
        Without<OpenXRLeftController>,
    )>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
    action_sets: Res<XrActionSets>,
) {
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
    //get left controller
    let left_grip_space = controller.grip_space(Hand::Left);
    let left_aim_space = controller.aim_space(Hand::Left);
    let left_postion = left_grip_space.0.pose.position.to_vec3();
    //TODO figure out how to not get the entity multiple times
    let left_aim_pose = left_controller_query.get_single_mut();
    //set aim pose
    match left_aim_pose {
        Ok(left_entity) => match left_entity.1 {
            Some(mut pose) => {
                *pose = AimPose(Transform {
                    translation: left_aim_space.0.pose.position.to_vec3(),
                    rotation: left_aim_space.0.pose.orientation.to_quat(),
                    scale: Vec3::splat(1.0),
                });
            }
            None => (),
        },
        Err(_) => debug!("no left controlelr entity found"),
    }
    //set translation
    let left_translation = left_controller_query.get_single_mut();
    match left_translation {
        Ok(mut left_entity) => left_entity.0.translation = left_postion,
        Err(_) => (),
    }
    //set rotation
    let left_rotataion = left_controller_query.get_single_mut();
    match left_rotataion {
        Ok(mut left_entity) => {
            left_entity.0.rotation = left_grip_space.0.pose.orientation.to_quat()
        }
        Err(_) => (),
    }
    //get right controller
    let right_grip_space = controller.grip_space(Hand::Right);
    let right_aim_space = controller.aim_space(Hand::Right);
    let right_postion = right_grip_space.0.pose.position.to_vec3();

    let right_aim_pose = right_controller_query.get_single_mut();
    match right_aim_pose {
        Ok(right_entity) => match right_entity.1 {
            Some(mut pose) => {
                *pose = AimPose(Transform {
                    translation: right_aim_space.0.pose.position.to_vec3(),
                    rotation: right_aim_space.0.pose.orientation.to_quat(),
                    scale: Vec3::splat(1.0),
                });
            }
            None => (),
        },
        Err(_) => debug!("no right controlelr entity found"),
    }
    //set translation
    let right_translation = right_controller_query.get_single_mut();
    match right_translation {
        Ok(mut right_entity) => right_entity.0.translation = right_postion,
        Err(_) => (),
    }
    //set rotation
    let right_rotataion = right_controller_query.get_single_mut();
    match right_rotataion {
        Ok(mut right_entity) => {
            right_entity.0.rotation = right_grip_space.0.pose.orientation.to_quat()
        }
        Err(_) => (),
    }
}
