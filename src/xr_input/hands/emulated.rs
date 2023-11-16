use std::f32::consts::PI;

use bevy::prelude::*;
use openxr::{ActionTy, HandJoint};

use super::common::{get_bone_gizmo_style, HandBoneRadius};
use crate::{
    resources::{XrInstance, XrSession},
    xr_input::{
        actions::{
            ActionHandednes, ActionType, SetupActionSet, SetupActionSets, XrActionSets, XrBinding,
        },
        hand_poses::get_simulated_open_hand_transforms,
        trackers::{OpenXRLeftController, OpenXRRightController, OpenXRTrackingRoot},
        Hand,
    },
};

use super::{BoneTrackingStatus, HandBone};

pub enum TouchValue<T: ActionTy> {
    None,
    Touched(T),
}

pub struct HandEmulationPlugin;

impl Plugin for HandEmulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_hand_skeleton_from_emulated);
        app.add_systems(Startup, setup_hand_emulation_action_set);
    }
}

const HAND_ACTION_SET: &str = "hand_pose_approx";

fn setup_hand_emulation_action_set(mut action_sets: ResMut<SetupActionSets>) {
    let mut action_set = action_sets.add_action_set(HAND_ACTION_SET, "Hand Pose Approximaiton", 0);
    action_set.new_action(
        "thumb_touch",
        "Thumb Touched",
        ActionType::Bool,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "thumb_x",
        "Thumb X",
        ActionType::F32,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "thumb_y",
        "Thumb Y",
        ActionType::F32,
        ActionHandednes::Double,
    );

    action_set.new_action(
        "index_touch",
        "Index Finger Touched",
        ActionType::Bool,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "index_value",
        "Index Finger Pull",
        ActionType::F32,
        ActionHandednes::Double,
    );

    action_set.new_action(
        "middle_value",
        "Middle Finger Pull",
        ActionType::F32,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "ring_value",
        "Ring Finger Pull",
        ActionType::F32,
        ActionHandednes::Double,
    );
    action_set.new_action(
        "little_value",
        "Little Finger Pull",
        ActionType::F32,
        ActionHandednes::Double,
    );

    suggest_oculus_touch_profile(&mut action_set);
}

pub struct EmulatedHandPoseData {}

fn suggest_oculus_touch_profile(action_set: &mut SetupActionSet) {
    action_set.suggest_binding(
        "/interaction_profiles/oculus/touch_controller",
        &[
            XrBinding::new("thumb_x", "/user/hand/left/input/thumbstick/x"),
            XrBinding::new("thumb_x", "/user/hand/right/input/thumbstick/x"),
            XrBinding::new("thumb_y", "/user/hand/left/input/thumbstick/y"),
            XrBinding::new("thumb_y", "/user/hand/right/input/thumbstick/y"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/thumbstick/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/thumbstick/touch"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/x/touch"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/y/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/a/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/b/touch"),
            XrBinding::new("thumb_touch", "/user/hand/left/input/thumbrest/touch"),
            XrBinding::new("thumb_touch", "/user/hand/right/input/thumbrest/touch"),
            XrBinding::new("index_touch", "/user/hand/left/input/trigger/touch"),
            XrBinding::new("index_value", "/user/hand/left/input/trigger/value"),
            XrBinding::new("index_touch", "/user/hand/right/input/trigger/touch"),
            XrBinding::new("index_value", "/user/hand/right/input/trigger/value"),
            XrBinding::new("middle_value", "/user/hand/left/input/squeeze/value"),
            XrBinding::new("middle_value", "/user/hand/right/input/squeeze/value"),
            XrBinding::new("ring_value", "/user/hand/left/input/squeeze/value"),
            XrBinding::new("ring_value", "/user/hand/right/input/squeeze/value"),
            XrBinding::new("little_value", "/user/hand/left/input/squeeze/value"),
            XrBinding::new("little_value", "/user/hand/right/input/squeeze/value"),
        ],
    );
}

pub(crate) fn update_hand_skeleton_from_emulated(
    session: Res<XrSession>,
    instance: Res<XrInstance>,
    action_sets: Res<XrActionSets>,
    left_controller_transform: Query<&Transform, With<OpenXRLeftController>>,
    right_controller_transform: Query<&Transform, With<OpenXRRightController>>,
    tracking_root_transform: Query<&Transform, With<OpenXRTrackingRoot>>,
    mut bones: Query<
        (
            &mut Transform,
            &HandBone,
            &Hand,
            &BoneTrackingStatus,
            &mut HandBoneRadius,
        ),
        (
            Without<OpenXRLeftController>,
            Without<OpenXRRightController>,
            Without<OpenXRTrackingRoot>,
        ),
    >,
) {
    let mut data: [[Transform; 26]; 2] = [[Transform::default(); 26]; 2];
    for (subaction_path, hand) in [
        (
            instance.string_to_path("/user/hand/left").unwrap(),
            Hand::Left,
        ),
        (
            instance.string_to_path("/user/hand/right").unwrap(),
            Hand::Right,
        ),
    ] {
        let thumb_curl = match action_sets
            .get_action_bool(HAND_ACTION_SET, "thumb_touch")
            .unwrap()
            .state(&session, subaction_path)
            .unwrap()
            .current_state
        {
            true => 1.0,
            false => 0.0,
        };
        let index_curl = action_sets
            .get_action_f32(HAND_ACTION_SET, "index_value")
            .unwrap()
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        let middle_curl = action_sets
            .get_action_f32(HAND_ACTION_SET, "middle_value")
            .unwrap()
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        let ring_curl = action_sets
            .get_action_f32(HAND_ACTION_SET, "ring_value")
            .unwrap()
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        let little_curl = action_sets
            .get_action_f32(HAND_ACTION_SET, "little_value")
            .unwrap()
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        data[match hand {
            Hand::Left => 0,
            Hand::Right => 1,
        }] = update_hand_bones_emulated(
            match hand {
                Hand::Left => left_controller_transform.single(),
                Hand::Right => right_controller_transform.single(),
            },
            hand,
            thumb_curl,
            index_curl,
            middle_curl,
            ring_curl,
            little_curl,
        );
    }
    let trt = tracking_root_transform.single();
    for (mut t, bone, hand, status, mut radius) in bones.iter_mut() {
        match status {
            BoneTrackingStatus::Emulated => {}
            BoneTrackingStatus::Tracked => continue,
        }
        radius.0 = get_bone_gizmo_style(bone).0;

        *t = data[match hand {
            Hand::Left => 0,
            Hand::Right => 1,
        }][bone.get_index_from_bone()];
        *t = t.with_scale(trt.scale);
        *t = t.with_rotation(trt.rotation * t.rotation);
        *t = t.with_translation(trt.transform_point(t.translation));
    }
}
pub fn update_hand_bones_emulated(
    controller_transform: &Transform,
    hand: Hand,
    thumb_curl: f32,
    index_curl: f32,
    middle_curl: f32,
    ring_curl: f32,
    little_curl: f32,
) -> [Transform; 26] {
    let left_hand_rot = Quat::from_rotation_y(PI);
    let hand_translation: Vec3 = controller_transform.translation;

    let controller_quat: Quat = match hand {
        Hand::Left => controller_transform.rotation.mul_quat(left_hand_rot),
        Hand::Right => controller_transform.rotation,
    };

    let splay_direction = match hand {
        Hand::Left => -1.0,
        Hand::Right => 1.0,
    };
    //lets make a structure to hold our calculated transforms for now
    let mut calc_transforms = [Transform::default(); 26];

    //get palm quat
    let y = Quat::from_rotation_y(-90.0 * PI / 180.0);
    let x = Quat::from_rotation_x(-90.0 * PI / 180.0);
    let palm_quat = controller_quat.mul_quat(y).mul_quat(x);
    //get simulated bones
    let hand_transform_array: [Transform; 26] = get_simulated_open_hand_transforms(hand);
    //palm
    let palm = hand_transform_array[HandJoint::PALM];
    calc_transforms[HandJoint::PALM] = Transform {
        translation: hand_translation + palm.translation,
        ..default()
    };
    //wrist
    let wrist = hand_transform_array[HandJoint::WRIST];
    calc_transforms[HandJoint::WRIST] = Transform {
        translation: hand_translation + palm.translation + palm_quat.mul_vec3(wrist.translation),
        ..default()
    };

    //thumb
    let thumb_joints = [
        HandJoint::THUMB_METACARPAL,
        HandJoint::THUMB_PROXIMAL,
        HandJoint::THUMB_DISTAL,
        HandJoint::THUMB_TIP,
    ];
    let mut prior_start: Option<Vec3> = None;
    let mut prior_quat: Option<Quat> = None;
    let mut prior_vector: Option<Vec3> = None;
    let splay = Quat::from_rotation_y(splay_direction * 30.0 * PI / 180.0);
    let huh = Quat::from_rotation_x(-35.0 * PI / 180.0);
    let splay_quat = palm_quat.mul_quat(huh).mul_quat(splay);
    for bone in thumb_joints.iter() {
        match prior_start {
            Some(start) => {
                let curl_angle: f32 = get_bone_curl_angle(*bone, thumb_curl);
                let tp_lrot = Quat::from_rotation_y(splay_direction * curl_angle * PI / 180.0);
                let tp_quat = prior_quat.unwrap().mul_quat(tp_lrot);
                let thumb_prox = hand_transform_array[*bone];
                let tp_start = start + prior_vector.unwrap();
                let tp_vector = tp_quat.mul_vec3(thumb_prox.translation);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tp_start + tp_vector,
                    ..default()
                };
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tm_start + tm_vector,
                    ..default()
                };
            }
        }
    }

    //index
    let thumb_joints = [
        HandJoint::INDEX_METACARPAL,
        HandJoint::INDEX_PROXIMAL,
        HandJoint::INDEX_INTERMEDIATE,
        HandJoint::INDEX_DISTAL,
        HandJoint::INDEX_TIP,
    ];
    let mut prior_start: Option<Vec3> = None;
    let mut prior_quat: Option<Quat> = None;
    let mut prior_vector: Option<Vec3> = None;
    let splay = Quat::from_rotation_y(splay_direction * 10.0 * PI / 180.0);
    let splay_quat = palm_quat.mul_quat(splay);
    for bone in thumb_joints.iter() {
        match prior_start {
            Some(start) => {
                let curl_angle: f32 = get_bone_curl_angle(*bone, index_curl);
                let tp_lrot = Quat::from_rotation_x(curl_angle * PI / 180.0);
                let tp_quat = prior_quat.unwrap().mul_quat(tp_lrot);
                let thumb_prox = hand_transform_array[*bone];
                let tp_start = start + prior_vector.unwrap();
                let tp_vector = tp_quat.mul_vec3(thumb_prox.translation);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tp_start + tp_vector,
                    ..default()
                };
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tm_start + tm_vector,
                    ..default()
                };
            }
        }
    }

    //middle
    let thumb_joints = [
        HandJoint::MIDDLE_METACARPAL,
        HandJoint::MIDDLE_PROXIMAL,
        HandJoint::MIDDLE_INTERMEDIATE,
        HandJoint::MIDDLE_DISTAL,
        HandJoint::MIDDLE_TIP,
    ];
    let mut prior_start: Option<Vec3> = None;
    let mut prior_quat: Option<Quat> = None;
    let mut prior_vector: Option<Vec3> = None;
    let splay = Quat::from_rotation_y(splay_direction * 0.0 * PI / 180.0);
    let splay_quat = palm_quat.mul_quat(splay);
    for bone in thumb_joints.iter() {
        match prior_start {
            Some(start) => {
                let curl_angle: f32 = get_bone_curl_angle(*bone, middle_curl);
                let tp_lrot = Quat::from_rotation_x(curl_angle * PI / 180.0);
                let tp_quat = prior_quat.unwrap().mul_quat(tp_lrot);
                let thumb_prox = hand_transform_array[*bone];
                let tp_start = start + prior_vector.unwrap();
                let tp_vector = tp_quat.mul_vec3(thumb_prox.translation);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tp_start + tp_vector,
                    ..default()
                };
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tm_start + tm_vector,
                    ..default()
                };
            }
        }
    }
    //ring
    let thumb_joints = [
        HandJoint::RING_METACARPAL,
        HandJoint::RING_PROXIMAL,
        HandJoint::RING_INTERMEDIATE,
        HandJoint::RING_DISTAL,
        HandJoint::RING_TIP,
    ];
    let mut prior_start: Option<Vec3> = None;
    let mut prior_quat: Option<Quat> = None;
    let mut prior_vector: Option<Vec3> = None;
    let splay = Quat::from_rotation_y(splay_direction * -10.0 * PI / 180.0);
    let splay_quat = palm_quat.mul_quat(splay);
    for bone in thumb_joints.iter() {
        match prior_start {
            Some(start) => {
                let curl_angle: f32 = get_bone_curl_angle(*bone, ring_curl);
                let tp_lrot = Quat::from_rotation_x(curl_angle * PI / 180.0);
                let tp_quat = prior_quat.unwrap().mul_quat(tp_lrot);
                let thumb_prox = hand_transform_array[*bone];
                let tp_start = start + prior_vector.unwrap();
                let tp_vector = tp_quat.mul_vec3(thumb_prox.translation);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tp_start + tp_vector,
                    ..default()
                };
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tm_start + tm_vector,
                    ..default()
                };
            }
        }
    }

    //little
    let thumb_joints = [
        HandJoint::LITTLE_METACARPAL,
        HandJoint::LITTLE_PROXIMAL,
        HandJoint::LITTLE_INTERMEDIATE,
        HandJoint::LITTLE_DISTAL,
        HandJoint::LITTLE_TIP,
    ];
    let mut prior_start: Option<Vec3> = None;
    let mut prior_quat: Option<Quat> = None;
    let mut prior_vector: Option<Vec3> = None;
    let splay = Quat::from_rotation_y(splay_direction * -20.0 * PI / 180.0);
    let splay_quat = palm_quat.mul_quat(splay);
    for bone in thumb_joints.iter() {
        match prior_start {
            Some(start) => {
                let curl_angle: f32 = get_bone_curl_angle(*bone, little_curl);
                let tp_lrot = Quat::from_rotation_x(curl_angle * PI / 180.0);
                let tp_quat = prior_quat.unwrap().mul_quat(tp_lrot);
                let thumb_prox = hand_transform_array[*bone];
                let tp_start = start + prior_vector.unwrap();
                let tp_vector = tp_quat.mul_vec3(thumb_prox.translation);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tp_start + tp_vector,
                    ..default()
                };
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
                //store it
                calc_transforms[*bone] = Transform {
                    translation: tm_start + tm_vector,
                    ..default()
                };
            }
        }
    }
    calc_transforms
}

fn get_bone_curl_angle(bone: HandJoint, curl: f32) -> f32 {
    let mul: f32 = match bone {
        HandJoint::INDEX_PROXIMAL => 0.0,
        HandJoint::MIDDLE_PROXIMAL => 0.0,
        HandJoint::RING_PROXIMAL => 0.0,
        HandJoint::LITTLE_PROXIMAL => 0.0,
        HandJoint::THUMB_PROXIMAL => 0.0,
        _ => 1.0,
    };
    let curl_angle = -((mul * curl * 80.0) + 5.0);
    return curl_angle;
}
