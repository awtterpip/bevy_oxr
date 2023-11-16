use std::f32::consts::PI;

use bevy::prelude::{
    default, info, Color, Commands, Component, Deref, DerefMut, Entity, Gizmos, GlobalTransform,
    Plugin, PostUpdate, PreUpdate, Quat, Query, Res, ResMut, Resource, SpatialBundle, Startup,
    Transform, Update, Vec3, With, Without,
};
use openxr::{HandJoint, Posef};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
    xr_input::Vec3Conv,
};

use super::{
    actions::XrActionSets,
    hand_poses::get_simulated_open_hand_transforms,
    handtracking::HandTrackingTracker,
    oculus_touch::OculusController,
    trackers::{OpenXRLeftController, OpenXRRightController, OpenXRTracker, OpenXRTrackingRoot},
    Hand, QuatConv, hands::{HandBone, BoneTrackingStatus},
};

/// add debug renderer for controllers
#[derive(Default)]
pub struct OpenXrHandInput;

impl Plugin for OpenXrHandInput {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // .add_systems(Update, update_hand_skeletons)
            // .add_systems(PreUpdate, update_hand_states)
            .add_systems(Startup, spawn_hand_entities);
            // .insert_resource(HandStatesResource::default())
            // .insert_resource(HandInputSource::default());
    }
}

/// add debug renderer for controllers
#[derive(Default)]
pub struct HandInputDebugRenderer;

impl Plugin for HandInputDebugRenderer {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostUpdate, draw_hand_entities);
    }
}

#[derive(Resource)]
pub enum HandInputSource {
    Emulated,
    OpenXr,
}

impl Default for HandInputSource {
    fn default() -> Self {
        HandInputSource::OpenXr
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct HandsResource {
    pub left: HandResource,
    pub right: HandResource,
}
#[derive(Clone, Copy)]
pub struct HandResource {
    pub palm: Entity,
    pub wrist: Entity,
    pub thumb: ThumbResource,
    pub index: IndexResource,
    pub middle: MiddleResource,
    pub ring: RingResource,
    pub little: LittleResource,
}

impl Default for HandResource {
    fn default() -> Self {
        Self {
            palm: Entity::PLACEHOLDER,
            wrist: Entity::PLACEHOLDER,
            thumb: Default::default(),
            index: Default::default(),
            middle: Default::default(),
            ring: Default::default(),
            little: Default::default(),
        }
    }
}
#[derive(Clone, Copy)]
pub struct ThumbResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for ThumbResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct IndexResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for IndexResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct MiddleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for MiddleResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct RingResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for RingResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct LittleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for LittleResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}

pub fn spawn_hand_entities(mut commands: Commands) {
    let hands = [Hand::Left, Hand::Right];
    let bones = HandBone::get_all_bones();
    //hand resource
    let mut hand_resource = HandsResource { ..default() };
    for hand in hands.iter() {
        for bone in bones.iter() {
            let boneid = commands
                .spawn((
                    SpatialBundle::default(),
                    bone.clone(),
                    OpenXRTracker,
                    hand.clone(),
                    BoneTrackingStatus::Emulated,
                ))
                .id();
            match hand {
                Hand::Left => match bone {
                    HandBone::Palm => hand_resource.left.palm = boneid,
                    HandBone::Wrist => hand_resource.left.wrist = boneid,
                    HandBone::ThumbMetacarpal => hand_resource.left.thumb.metacarpal = boneid,
                    HandBone::ThumbProximal => hand_resource.left.thumb.proximal = boneid,
                    HandBone::ThumbDistal => hand_resource.left.thumb.distal = boneid,
                    HandBone::ThumbTip => hand_resource.left.thumb.tip = boneid,
                    HandBone::IndexMetacarpal => hand_resource.left.index.metacarpal = boneid,
                    HandBone::IndexProximal => hand_resource.left.index.proximal = boneid,
                    HandBone::IndexIntermediate => hand_resource.left.index.intermediate = boneid,
                    HandBone::IndexDistal => hand_resource.left.index.distal = boneid,
                    HandBone::IndexTip => hand_resource.left.index.tip = boneid,
                    HandBone::MiddleMetacarpal => hand_resource.left.middle.metacarpal = boneid,
                    HandBone::MiddleProximal => hand_resource.left.middle.proximal = boneid,
                    HandBone::MiddleIntermediate => hand_resource.left.middle.intermediate = boneid,
                    HandBone::MiddleDistal => hand_resource.left.middle.distal = boneid,
                    HandBone::MiddleTip => hand_resource.left.middle.tip = boneid,
                    HandBone::RingMetacarpal => hand_resource.left.ring.metacarpal = boneid,
                    HandBone::RingProximal => hand_resource.left.ring.proximal = boneid,
                    HandBone::RingIntermediate => hand_resource.left.ring.intermediate = boneid,
                    HandBone::RingDistal => hand_resource.left.ring.distal = boneid,
                    HandBone::RingTip => hand_resource.left.ring.tip = boneid,
                    HandBone::LittleMetacarpal => hand_resource.left.little.metacarpal = boneid,
                    HandBone::LittleProximal => hand_resource.left.little.proximal = boneid,
                    HandBone::LittleIntermediate => hand_resource.left.little.intermediate = boneid,
                    HandBone::LittleDistal => hand_resource.left.little.distal = boneid,
                    HandBone::LittleTip => hand_resource.left.little.tip = boneid,
                },
                Hand::Right => match bone {
                    HandBone::Palm => hand_resource.right.palm = boneid,
                    HandBone::Wrist => hand_resource.right.wrist = boneid,
                    HandBone::ThumbMetacarpal => hand_resource.right.thumb.metacarpal = boneid,
                    HandBone::ThumbProximal => hand_resource.right.thumb.proximal = boneid,
                    HandBone::ThumbDistal => hand_resource.right.thumb.distal = boneid,
                    HandBone::ThumbTip => hand_resource.right.thumb.tip = boneid,
                    HandBone::IndexMetacarpal => hand_resource.right.index.metacarpal = boneid,
                    HandBone::IndexProximal => hand_resource.right.index.proximal = boneid,
                    HandBone::IndexIntermediate => hand_resource.right.index.intermediate = boneid,
                    HandBone::IndexDistal => hand_resource.right.index.distal = boneid,
                    HandBone::IndexTip => hand_resource.right.index.tip = boneid,
                    HandBone::MiddleMetacarpal => hand_resource.right.middle.metacarpal = boneid,
                    HandBone::MiddleProximal => hand_resource.right.middle.proximal = boneid,
                    HandBone::MiddleIntermediate => {
                        hand_resource.right.middle.intermediate = boneid
                    }
                    HandBone::MiddleDistal => hand_resource.right.middle.distal = boneid,
                    HandBone::MiddleTip => hand_resource.right.middle.tip = boneid,
                    HandBone::RingMetacarpal => hand_resource.right.ring.metacarpal = boneid,
                    HandBone::RingProximal => hand_resource.right.ring.proximal = boneid,
                    HandBone::RingIntermediate => hand_resource.right.ring.intermediate = boneid,
                    HandBone::RingDistal => hand_resource.right.ring.distal = boneid,
                    HandBone::RingTip => hand_resource.right.ring.tip = boneid,
                    HandBone::LittleMetacarpal => hand_resource.right.little.metacarpal = boneid,
                    HandBone::LittleProximal => hand_resource.right.little.proximal = boneid,
                    HandBone::LittleIntermediate => {
                        hand_resource.right.little.intermediate = boneid
                    }
                    HandBone::LittleDistal => hand_resource.right.little.distal = boneid,
                    HandBone::LittleTip => hand_resource.right.little.tip = boneid,
                },
            }
        }
    }
    commands.insert_resource(hand_resource);
}


pub fn update_hand_states(
    oculus_controller: Res<OculusController>,
    hand_states_option: Option<ResMut<HandStatesResource>>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    action_sets: Res<XrActionSets>,
) {
    match hand_states_option {
        Some(mut hands) => {
            //lock frame
            let frame_state = *frame_state.lock().unwrap();
            //get controller
            let controller =
                oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);

            //right hand
            let squeeze = controller.squeeze(Hand::Right);
            let trigger_state = controller.trigger(Hand::Right);
            let calc_trigger_state = match controller.trigger_touched(Hand::Right) {
                true => match trigger_state > 0.0 {
                    true => TriggerState::PULLED,
                    false => TriggerState::TOUCHED,
                },
                false => TriggerState::OFF,
            };
            //button a
            let mut a_state = ButtonState::OFF;
            if controller.a_button_touched() {
                a_state = ButtonState::TOUCHED;
            }
            if controller.a_button() {
                a_state = ButtonState::PRESSED;
            }

            //button b
            let mut b_state = ButtonState::OFF;
            if controller.b_button_touched() {
                b_state = ButtonState::TOUCHED;
            }
            if controller.b_button() {
                b_state = ButtonState::PRESSED;
            }

            let thumbstick_state = controller.thumbstick(Hand::Right);
            let calc_thumbstick_state = match controller.thumbstick_touch(Hand::Right) {
                true => match thumbstick_state.x > 0.0 || thumbstick_state.y > 0.0 {
                    true => ThumbstickState::PRESSED,
                    false => ThumbstickState::TOUCHED,
                },
                false => ThumbstickState::OFF,
            };

            let right_state = HandState {
                grip: squeeze,
                trigger_state: calc_trigger_state,
                a_button: a_state,
                b_button: b_state,
                thumbstick: calc_thumbstick_state,
            };

            //left
            let squeeze = controller.squeeze(Hand::Left);
            let trigger_state = controller.trigger(Hand::Left);
            let calc_trigger_state = match controller.trigger_touched(Hand::Left) {
                true => match trigger_state > 0.0 {
                    true => TriggerState::PULLED,
                    false => TriggerState::TOUCHED,
                },
                false => TriggerState::OFF,
            };
            //button a
            let mut a_state = ButtonState::OFF;
            if controller.x_button_touched() {
                a_state = ButtonState::TOUCHED;
            }
            if controller.x_button() {
                a_state = ButtonState::PRESSED;
            }

            //button b
            let mut b_state = ButtonState::OFF;
            if controller.y_button_touched() {
                b_state = ButtonState::TOUCHED;
            }
            if controller.y_button() {
                b_state = ButtonState::PRESSED;
            }

            let thumbstick_state = controller.thumbstick(Hand::Left);
            let calc_thumbstick_state = match controller.thumbstick_touch(Hand::Left) {
                true => match thumbstick_state.x > 0.0 || thumbstick_state.y > 0.0 {
                    true => ThumbstickState::PRESSED,
                    false => ThumbstickState::TOUCHED,
                },
                false => ThumbstickState::OFF,
            };

            let left_state = HandState {
                grip: squeeze,
                trigger_state: calc_trigger_state,
                a_button: a_state,
                b_button: b_state,
                thumbstick: calc_thumbstick_state,
            };

            hands.left = left_state;
            hands.right = right_state;
        }
        None => info!("hand states resource not init yet"),
    }
}

#[derive(Clone, Copy)]
pub enum ButtonState {
    OFF,
    TOUCHED,
    PRESSED,
}

impl Default for ButtonState {
    fn default() -> Self {
        ButtonState::OFF
    }
}
#[derive(Clone, Copy)]
pub enum ThumbstickState {
    OFF,
    TOUCHED,
    PRESSED,
}

impl Default for ThumbstickState {
    fn default() -> Self {
        ThumbstickState::OFF
    }
}
#[derive(Clone, Copy)]
pub enum TriggerState {
    OFF,
    TOUCHED,
    PULLED,
}

impl Default for TriggerState {
    fn default() -> Self {
        TriggerState::OFF
    }
}

#[derive(Default, Resource)]
pub struct HandStatesResource {
    pub left: HandState,
    pub right: HandState,
}

#[derive(Clone, Copy)]
pub struct HandState {
    pub grip: f32,
    pub trigger_state: TriggerState,
    pub a_button: ButtonState,
    pub b_button: ButtonState,
    pub thumbstick: ThumbstickState,
}

impl HandState {
    pub fn get_index_curl(&self) -> f32 {
        match self.trigger_state {
            TriggerState::OFF => 0.0,
            TriggerState::TOUCHED => 0.50,
            TriggerState::PULLED => 1.0,
        }
    }

    pub fn get_thumb_curl(&self) -> f32 {
        match self.thumbstick {
            ThumbstickState::OFF => (),
            ThumbstickState::TOUCHED => return 0.25,
            ThumbstickState::PRESSED => return 0.25,
        };

        match self.a_button {
            ButtonState::OFF => (),
            ButtonState::TOUCHED => return 0.25,
            ButtonState::PRESSED => return 0.25,
        };

        match self.b_button {
            ButtonState::OFF => (),
            ButtonState::TOUCHED => return 0.25,
            ButtonState::PRESSED => return 0.25,
        };
        //if no thumb actions taken return open position
        return 0.0;
    }
}

impl Default for HandState {
    fn default() -> Self {
        Self {
            grip: Default::default(),
            trigger_state: Default::default(),
            a_button: Default::default(),
            b_button: Default::default(),
            thumbstick: Default::default(),
        }
    }
}

pub fn update_hand_bones_emulated(
    controller_transform: Transform,
    hand: Hand,
    hand_state: HandState,

    hand_bone_query: &mut Query<(
        Entity,
        &mut Transform,
        &HandBone,
        &Hand,
        Option<&mut HandBoneRadius>,
        Without<OpenXRTrackingRoot>,
    )>,
) {
    let left_hand_rot = Quat::from_rotation_y(180.0 * PI / 180.0);
    let hand_translation: Vec3 = match hand {
        Hand::Left => controller_transform.translation,
        Hand::Right => controller_transform.translation,
    };

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

    //curl represents how closed the hand is from 0 to 1;
    let grip_curl = hand_state.grip;
    let index_curl = hand_state.get_index_curl();
    let thumb_curl = hand_state.get_thumb_curl();
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
                let curl_angle: f32 = get_bone_curl_angle(*bone, grip_curl);
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
                let curl_angle: f32 = get_bone_curl_angle(*bone, grip_curl);
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
                let curl_angle: f32 = get_bone_curl_angle(*bone, grip_curl);
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

    //now that we have all the transforms lets assign them
    for (_, mut transform, handbone, bonehand, _, _) in hand_bone_query.iter_mut() {
        if *bonehand == hand {
            //if the hands match lets go
            let index = match_index(handbone);
            *transform = calc_transforms[index];
        }
    }
}

fn match_index(handbone: &HandBone) -> HandJoint {
    match handbone {
        HandBone::Palm => HandJoint::PALM,
        HandBone::Wrist => HandJoint::WRIST,
        HandBone::ThumbMetacarpal => HandJoint::THUMB_METACARPAL,
        HandBone::ThumbProximal => HandJoint::THUMB_PROXIMAL,
        HandBone::ThumbDistal => HandJoint::THUMB_DISTAL,
        HandBone::ThumbTip => HandJoint::THUMB_TIP,
        HandBone::IndexMetacarpal => HandJoint::INDEX_METACARPAL,
        HandBone::IndexProximal => HandJoint::INDEX_PROXIMAL,
        HandBone::IndexIntermediate => HandJoint::INDEX_INTERMEDIATE,
        HandBone::IndexDistal => HandJoint::INDEX_DISTAL,
        HandBone::IndexTip => HandJoint::INDEX_TIP,
        HandBone::MiddleMetacarpal => HandJoint::MIDDLE_METACARPAL,
        HandBone::MiddleProximal => HandJoint::MIDDLE_PROXIMAL,
        HandBone::MiddleIntermediate => HandJoint::MIDDLE_INTERMEDIATE,
        HandBone::MiddleDistal => HandJoint::MIDDLE_DISTAL,
        HandBone::MiddleTip => HandJoint::MIDDLE_TIP,
        HandBone::RingMetacarpal => HandJoint::RING_METACARPAL,
        HandBone::RingProximal => HandJoint::RING_PROXIMAL,
        HandBone::RingIntermediate => HandJoint::RING_INTERMEDIATE,
        HandBone::RingDistal => HandJoint::RING_DISTAL,
        HandBone::RingTip => HandJoint::RING_TIP,
        HandBone::LittleMetacarpal => HandJoint::LITTLE_METACARPAL,
        HandBone::LittleProximal => HandJoint::LITTLE_PROXIMAL,
        HandBone::LittleIntermediate => HandJoint::LITTLE_INTERMEDIATE,
        HandBone::LittleDistal => HandJoint::LITTLE_DISTAL,
        HandBone::LittleTip => HandJoint::LITTLE_TIP,
    }
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

fn log_hand(hand_pose: [Posef; 26]) {
    let _palm_wrist = hand_pose[HandJoint::WRIST].position.to_vec3()
        - hand_pose[HandJoint::PALM].position.to_vec3();
    info!(
        "palm-wrist: {}",
        hand_pose[HandJoint::WRIST].position.to_vec3()
            - hand_pose[HandJoint::PALM].position.to_vec3()
    );

    info!(
        "wrist-tm: {}",
        hand_pose[HandJoint::THUMB_METACARPAL].position.to_vec3()
            - hand_pose[HandJoint::WRIST].position.to_vec3()
    );
    info!(
        "tm-tp: {}",
        hand_pose[HandJoint::THUMB_PROXIMAL].position.to_vec3()
            - hand_pose[HandJoint::THUMB_METACARPAL].position.to_vec3()
    );
    info!(
        "tp-td: {}",
        hand_pose[HandJoint::THUMB_DISTAL].position.to_vec3()
            - hand_pose[HandJoint::THUMB_PROXIMAL].position.to_vec3()
    );
    info!(
        "td-tt: {}",
        hand_pose[HandJoint::THUMB_TIP].position.to_vec3()
            - hand_pose[HandJoint::THUMB_DISTAL].position.to_vec3()
    );

    info!(
        "wrist-im: {}",
        hand_pose[HandJoint::INDEX_METACARPAL].position.to_vec3()
            - hand_pose[HandJoint::WRIST].position.to_vec3()
    );
    info!(
        "im-ip: {}",
        hand_pose[HandJoint::INDEX_PROXIMAL].position.to_vec3()
            - hand_pose[HandJoint::INDEX_METACARPAL].position.to_vec3()
    );
    info!(
        "ip-ii: {}",
        hand_pose[HandJoint::INDEX_INTERMEDIATE].position.to_vec3()
            - hand_pose[HandJoint::INDEX_PROXIMAL].position.to_vec3()
    );
    info!(
        "ii-id: {}",
        hand_pose[HandJoint::INDEX_DISTAL].position.to_vec3()
            - hand_pose[HandJoint::INDEX_INTERMEDIATE].position.to_vec3()
    );
    info!(
        "id-it: {}",
        hand_pose[HandJoint::INDEX_TIP].position.to_vec3()
            - hand_pose[HandJoint::INDEX_DISTAL].position.to_vec3()
    );

    info!(
        "wrist-mm: {}",
        hand_pose[HandJoint::MIDDLE_METACARPAL].position.to_vec3()
            - hand_pose[HandJoint::WRIST].position.to_vec3()
    );
    info!(
        "mm-mp: {}",
        hand_pose[HandJoint::MIDDLE_PROXIMAL].position.to_vec3()
            - hand_pose[HandJoint::MIDDLE_METACARPAL].position.to_vec3()
    );
    info!(
        "mp-mi: {}",
        hand_pose[HandJoint::MIDDLE_INTERMEDIATE].position.to_vec3()
            - hand_pose[HandJoint::MIDDLE_PROXIMAL].position.to_vec3()
    );
    info!(
        "mi-md: {}",
        hand_pose[HandJoint::MIDDLE_DISTAL].position.to_vec3()
            - hand_pose[HandJoint::MIDDLE_INTERMEDIATE].position.to_vec3()
    );
    info!(
        "md-mt: {}",
        hand_pose[HandJoint::MIDDLE_TIP].position.to_vec3()
            - hand_pose[HandJoint::MIDDLE_DISTAL].position.to_vec3()
    );

    info!(
        "wrist-rm: {}",
        hand_pose[HandJoint::RING_METACARPAL].position.to_vec3()
            - hand_pose[HandJoint::WRIST].position.to_vec3()
    );
    info!(
        "rm-rp: {}",
        hand_pose[HandJoint::RING_PROXIMAL].position.to_vec3()
            - hand_pose[HandJoint::RING_METACARPAL].position.to_vec3()
    );
    info!(
        "rp-ri: {}",
        hand_pose[HandJoint::RING_INTERMEDIATE].position.to_vec3()
            - hand_pose[HandJoint::RING_PROXIMAL].position.to_vec3()
    );
    info!(
        "ri-rd: {}",
        hand_pose[HandJoint::RING_DISTAL].position.to_vec3()
            - hand_pose[HandJoint::RING_INTERMEDIATE].position.to_vec3()
    );
    info!(
        "rd-rt: {}",
        hand_pose[HandJoint::RING_TIP].position.to_vec3()
            - hand_pose[HandJoint::RING_DISTAL].position.to_vec3()
    );

    info!(
        "wrist-lm: {}",
        hand_pose[HandJoint::LITTLE_METACARPAL].position.to_vec3()
            - hand_pose[HandJoint::WRIST].position.to_vec3()
    );
    info!(
        "lm-lp: {}",
        hand_pose[HandJoint::LITTLE_PROXIMAL].position.to_vec3()
            - hand_pose[HandJoint::LITTLE_METACARPAL].position.to_vec3()
    );
    info!(
        "lp-li: {}",
        hand_pose[HandJoint::LITTLE_INTERMEDIATE].position.to_vec3()
            - hand_pose[HandJoint::LITTLE_PROXIMAL].position.to_vec3()
    );
    info!(
        "li-ld: {}",
        hand_pose[HandJoint::LITTLE_DISTAL].position.to_vec3()
            - hand_pose[HandJoint::LITTLE_INTERMEDIATE].position.to_vec3()
    );
    info!(
        "ld-lt: {}",
        hand_pose[HandJoint::LITTLE_TIP].position.to_vec3()
            - hand_pose[HandJoint::LITTLE_DISTAL].position.to_vec3()
    );
}

pub fn update_hand_skeletons(
    tracking_root_query: Query<(&Transform, With<OpenXRTrackingRoot>)>,
    right_controller_query: Query<(&GlobalTransform, With<OpenXRRightController>)>,
    left_controller_query: Query<(&GlobalTransform, With<OpenXRLeftController>)>,
    hand_states_option: Option<ResMut<HandStatesResource>>,
    mut commands: Commands,
    mut hand_bone_query: Query<(
        Entity,
        &mut Transform,
        &HandBone,
        &Hand,
        Option<&mut HandBoneRadius>,
        Without<OpenXRTrackingRoot>,
    )>,
    input_source: Option<Res<HandInputSource>>,
    hand_tracking: Option<Res<HandTrackingTracker>>,
    xr_input: Res<XrInput>,
    xr_frame_state: Res<XrFrameState>,
) {
    match input_source {
        Some(res) => match *res {
            HandInputSource::Emulated => {
                // info!("hand input source is emulated");
                match hand_states_option {
                    Some(hands) => {
                        let left_hand_transform = left_controller_query
                            .get_single()
                            .unwrap()
                            .0
                            .compute_transform();
                        update_hand_bones_emulated(
                            left_hand_transform,
                            Hand::Left,
                            hands.left,
                            &mut hand_bone_query,
                        );
                        let right_hand_transform = right_controller_query
                            .get_single()
                            .unwrap()
                            .0
                            .compute_transform();
                        update_hand_bones_emulated(
                            right_hand_transform,
                            Hand::Right,
                            hands.right,
                            &mut hand_bone_query,
                        );
                    }
                    None => info!("hand states resource not initialized yet"),
                }
            }
            HandInputSource::OpenXr => match hand_tracking {
                Some(tracking) => {
                    let hand_ref = tracking.get_ref(&xr_input, &xr_frame_state);
                    let (root_transform, _) = tracking_root_query.get_single().unwrap();
                    let left_data = hand_ref.get_left_poses();
                    let right_data = hand_ref.get_right_poses();

                    for (entity, mut transform, bone, hand, radius, _) in hand_bone_query.iter_mut()
                    {
                        let bone_data = match (hand, left_data, right_data) {
                            (Hand::Left, Some(data), _) => data[bone.get_index_from_bone()],
                            (Hand::Right, _, Some(data)) => data[bone.get_index_from_bone()],
                            _ => continue,
                        };
                        match radius {
                            Some(mut r) => r.0 = bone_data.radius,
                            None => {
                                commands
                                    .entity(entity)
                                    .insert(HandBoneRadius(bone_data.radius));
                            }
                        }
                        *transform = transform
                            .with_translation(
                                root_transform.transform_point(bone_data.pose.position.to_vec3()),
                            )
                            .with_rotation(
                                root_transform.rotation * bone_data.pose.orientation.to_quat(),
                            )
                    }
                }
                None => {}
            },
        },
        None => {
            info!("hand input source not initialized");
            return;
        }
    }
}

#[derive(Debug, Component, DerefMut, Deref)]
pub struct HandBoneRadius(pub f32);

pub fn draw_hand_entities(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &HandBone, Option<&HandBoneRadius>)>,
) {
    for (transform, hand_bone, hand_bone_radius) in query.iter() {
        let (radius, color) = get_bone_gizmo_style(hand_bone);
        gizmos.sphere(
            transform.translation,
            transform.rotation,
            hand_bone_radius.map_or(radius, |r| r.0),
            color,
        );
    }
}

fn get_bone_gizmo_style(hand_bone: &HandBone) -> (f32, Color) {
    match hand_bone {
        HandBone::Palm => (0.01, Color::WHITE),
        HandBone::Wrist => (0.01, Color::GRAY),
        HandBone::ThumbMetacarpal => (0.01, Color::RED),
        HandBone::ThumbProximal => (0.008, Color::RED),
        HandBone::ThumbDistal => (0.006, Color::RED),
        HandBone::ThumbTip => (0.004, Color::RED),
        HandBone::IndexMetacarpal => (0.01, Color::ORANGE),
        HandBone::IndexProximal => (0.008, Color::ORANGE),
        HandBone::IndexIntermediate => (0.006, Color::ORANGE),
        HandBone::IndexDistal => (0.004, Color::ORANGE),
        HandBone::IndexTip => (0.002, Color::ORANGE),
        HandBone::MiddleMetacarpal => (0.01, Color::YELLOW),
        HandBone::MiddleProximal => (0.008, Color::YELLOW),
        HandBone::MiddleIntermediate => (0.006, Color::YELLOW),
        HandBone::MiddleDistal => (0.004, Color::YELLOW),
        HandBone::MiddleTip => (0.002, Color::YELLOW),
        HandBone::RingMetacarpal => (0.01, Color::GREEN),
        HandBone::RingProximal => (0.008, Color::GREEN),
        HandBone::RingIntermediate => (0.006, Color::GREEN),
        HandBone::RingDistal => (0.004, Color::GREEN),
        HandBone::RingTip => (0.002, Color::GREEN),
        HandBone::LittleMetacarpal => (0.01, Color::BLUE),
        HandBone::LittleProximal => (0.008, Color::BLUE),
        HandBone::LittleIntermediate => (0.006, Color::BLUE),
        HandBone::LittleDistal => (0.004, Color::BLUE),
        HandBone::LittleTip => (0.002, Color::BLUE),
    }
}
