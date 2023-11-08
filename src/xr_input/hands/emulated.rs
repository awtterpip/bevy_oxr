use std::f32::consts::PI;

use bevy::prelude::*;
use openxr::{Action, ActionTy, Binding, HandJoint};

use crate::{
    resources::{XrInstance, XrSession},
    xr_input::{
        controllers::Touchable,
        hand_poses::get_simulated_open_hand_transforms,
        oculus_touch::ActionSets,
        trackers::{OpenXRLeftController, OpenXRRightController},
        Hand,
    },
};

use super::HandBone;

pub enum TouchValue<T: ActionTy> {
    None,
    Touched(T),
}

// #[derive(Deref, DerefMut, Resource)]
// pub struct EmulatedHandPoseFunctions {
//     pub get_base_pose: Box<dyn Fn(Hand) -> [Transform; 26] + Send + Sync>,
//     pub map_data: Box<dyn Fn(Hand) -> [Transform; 26] + Send + Sync>,
// }

pub struct EmulatedHandsPlugin;

impl Plugin for EmulatedHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_hand_skeleton_from_emulated);
        app.add_systems(
            Startup,
            setup_hand_emulation_action_set.map(|res| res.unwrap()),
        );
    }
}
#[derive(Resource)]
pub struct HandEmulationActionSet {
    thumb_touch: Action<bool>,
    thumb_x: Action<f32>,
    thumb_y: Action<f32>,
    index: Touchable<f32>,
    middle: Touchable<f32>,
    ring: Touchable<f32>,
    little: Touchable<f32>,
}

fn setup_hand_emulation_action_set(
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut action_sets: ResMut<ActionSets>,
    mut commands: Commands,
) -> anyhow::Result<()> {
    let left_path = instance.string_to_path("/user/hand/left").unwrap();
    let right_path = instance.string_to_path("/user/hand/right").unwrap();
    let hands = [left_path, right_path];
    // This unwrap Should not trigger since both strings are not empty
    let action_set = instance
        .create_action_set("hand_pose_approximation_set", "Hand Pose Approximaiton", 0)
        .unwrap();
    let hand_action_set = HandEmulationActionSet {
        thumb_touch: action_set.create_action::<bool>("thumb_touch", "Thumb Touched", &hands)?,
        thumb_x: action_set.create_action::<f32>("thumb_x", "Thumb X", &hands)?,
        thumb_y: action_set.create_action::<f32>("thumb_y", "Thumb Y", &hands)?,

        index: Touchable::<f32> {
            inner: action_set.create_action("index_value", "Index Finger Pull", &hands)?,
            touch: action_set.create_action("index_touch", "Index Finger Touch", &hands)?,
        },
        middle: Touchable::<f32> {
            inner: action_set.create_action("middle_value", "Middle Finger Pull", &hands)?,
            touch: action_set.create_action("middle_touch", "Middle Finger Touch", &hands)?,
        },
        ring: Touchable::<f32> {
            inner: action_set.create_action("ring_value", "Ring Finger Pull", &hands)?,
            touch: action_set.create_action("ring_touch", "Ring Finger Touch", &hands)?,
        },
        little: Touchable::<f32> {
            inner: action_set.create_action("little_value", "Little Finger Pull", &hands)?,
            touch: action_set.create_action("little_touch", "Little Finger Touch", &hands)?,
        },
    };

    suggest_oculus_touch_profile(&instance, &hand_action_set)?;

    session.attach_action_sets(&[&action_set])?;

    action_sets.0.push(action_set);

    commands.insert_resource(hand_action_set);

    Ok(())
}

pub struct EmulatedHandPoseData {}

fn bind<'a, T: ActionTy>(
    action: &'a Action<T>,
    path: &str,
    i: &XrInstance,
    bindings: &mut Vec<Binding<'a>>,
) -> anyhow::Result<()> {
    bindings.push(Binding::new(
        &action,
        i.string_to_path(&("/user/hand/left/input".to_string() + path))?,
    ));
    bindings.push(Binding::new(
        &action,
        i.string_to_path(&("/user/hand/right/input".to_string() + path))?,
    ));
    Ok(())
}
fn bind_single<'a, T: ActionTy>(
    action: &'a Action<T>,
    path: &str,
    hand: Hand,
    i: &XrInstance,
    bindings: &mut Vec<Binding<'a>>,
) -> anyhow::Result<()> {
    match hand {
        Hand::Left => bindings.push(Binding::new(
            &action,
            i.string_to_path(&("/user/hand/left/input".to_string() + path))?,
        )),
        Hand::Right => bindings.push(Binding::new(
            &action,
            i.string_to_path(&("/user/hand/right/input".to_string() + path))?,
        )),
    }
    Ok(())
}

fn suggest_oculus_touch_profile(
    i: &XrInstance,
    action_set: &HandEmulationActionSet,
) -> anyhow::Result<()> {
    let mut b: Vec<Binding> = Vec::new();
    bind(&action_set.thumb_x, "/thumbstick/x", i, &mut b)?;
    bind(&action_set.thumb_y, "/thumbstick/y", i, &mut b)?;
    bind(&action_set.thumb_touch, "/thumbstick/touch", i, &mut b)?;
    bind(&action_set.thumb_touch, "/thumbrest/touch", i, &mut b)?;
    // bind_single(&action_set.thumb_touch, "/x/touch", Hand::Left, i, &mut b)?;
    // bind_single(&action_set.thumb_touch, "/y/touch", Hand::Left, i, &mut b)?;
    // bind_single(&action_set.thumb_touch, "/a/touch", Hand::Right, i, &mut b)?;
    // bind_single(&action_set.thumb_touch, "/b/touch", Hand::Right, i, &mut b)?;
    
    // bind(&action_set.index.touch, "/trigger/touch", i, &mut b)?;
    // bind(&action_set.index.inner, "/trigger/value", i, &mut b)?;
    //
    // bind(&action_set.middle.touch, "/squeeze/touch", i, &mut b)?;
    // bind(&action_set.middle.inner, "/squeeze/value", i, &mut b)?;
    // bind(&action_set.ring.touch, "/squeeze/touch", i, &mut b)?;
    // bind(&action_set.ring.inner, "/squeeze/value", i, &mut b)?;
    // bind(&action_set.little.touch, "/squeeze/touch", i, &mut b)?;
    // bind(&action_set.little.inner, "/squeeze/value", i, &mut b)?;

    i.suggest_interaction_profile_bindings(
        i.string_to_path("/interaction_profiles/oculus/touch_controller")?,
        &b,
    )?;
    Ok(())
}

pub(crate) fn update_hand_skeleton_from_emulated(
    session: Res<XrSession>,
    instance: Res<XrInstance>,
    action_set: Res<HandEmulationActionSet>,
    left_controller_transform: Query<&Transform, With<OpenXRLeftController>>,
    right_controller_transform: Query<&Transform, With<OpenXRRightController>>,
    mut bones: Query<
        (&mut Transform, &HandBone, &Hand),
        (
            Without<OpenXRLeftController>,
            Without<OpenXRRightController>,
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
        let thumb_curl = match action_set
            .thumb_touch
            .state(&session, subaction_path)
            .unwrap()
            .current_state
        {
            true => 1.0,
            false => 0.0,
        };
        let index_curl = action_set
            .index
            .inner
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        let middle_curl = action_set
            .middle
            .inner
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        let ring_curl = action_set
            .ring
            .inner
            .state(&session, subaction_path)
            .unwrap()
            .current_state;
        let little_curl = action_set
            .little
            .inner
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
    for (mut t, bone, hand) in bones.iter_mut() {
        *t = data[match hand {
            Hand::Left => 0,
            Hand::Right => 1,
        }][bone.get_index_from_bone()]
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

fn get_bone_curl_angle(bone: HandJoint, thumb_curl: f32) -> f32 {
    todo!()
}
