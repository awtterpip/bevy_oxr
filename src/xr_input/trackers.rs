use bevy::prelude::{
    info, Added, BuildChildren, Commands, Component, Entity, Query, Res, Transform, Vec3, With,
    Without,
};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
};

use super::{oculus_touch::OculusController, Hand, QuatConv, Vec3Conv};

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
    instance: Res<XrInstance>,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
) {
    //lock dat frame?
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);
    //get left controller
    let left_grip_space = controller.grip_space(Hand::Left);
    let left_aim_space = controller.aim_space(Hand::Left);
    let left_postion = left_grip_space.0.pose.position.to_vec3();
    let left_aim_pose = left_controller_query.get_single_mut().unwrap().1;
    match left_aim_pose {
        Some(mut pose) => {
            *pose = AimPose(Transform {
                translation: left_aim_space.0.pose.position.to_vec3(),
                rotation: left_aim_space.0.pose.orientation.to_quat(),
                scale: Vec3::splat(1.0),
            });
        }
        None => (),
    }

    left_controller_query
        .get_single_mut()
        .unwrap()
        .0
        .translation = left_postion;

    left_controller_query.get_single_mut().unwrap().0.rotation =
        left_grip_space.0.pose.orientation.to_quat();
    //get right controller
    let right_grip_space = controller.grip_space(Hand::Right);
    let right_aim_space = controller.aim_space(Hand::Right);
    let right_postion = right_grip_space.0.pose.position.to_vec3();

    let right_aim_pose = right_controller_query.get_single_mut().unwrap().1;
    match right_aim_pose {
        Some(mut pose) => {
            *pose = AimPose(Transform {
                translation: right_aim_space.0.pose.position.to_vec3(),
                rotation: right_aim_space.0.pose.orientation.to_quat(),
                scale: Vec3::splat(1.0),
            });
        }
        None => (),
    }

    right_controller_query
        .get_single_mut()
        .unwrap()
        .0
        .translation = right_postion;

    right_controller_query.get_single_mut().unwrap().0.rotation =
        right_grip_space.0.pose.orientation.to_quat();
}
