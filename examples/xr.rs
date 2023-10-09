<<<<<<< HEAD

=======
use std::f32::consts::PI;
use std::ops::Mul;
>>>>>>> 68cdf19 (both hands work)

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::prelude::*;
use bevy::transform::components::Transform;
<<<<<<< HEAD
<<<<<<< HEAD

use bevy_openxr::xr_input::{QuatConv, Vec3Conv};
use bevy_openxr::xr_input::hand::{OpenXrHandInput, HandInputDebugRenderer};
=======
use bevy_openxr::xr_input::{Vec3Conv, QuatConv, Hand};
use bevy_openxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
>>>>>>> 68cdf19 (both hands work)
=======
use bevy::{gizmos, prelude::*};
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession};

use bevy_openxr::xr_input::hand_poses::*;
use bevy_openxr::xr_input::oculus_touch::OculusController;
>>>>>>> 319a2dc (hand state resource is used to drive skeleton)
use bevy_openxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_openxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
};
use bevy_openxr::xr_input::{Hand, Vec3Conv};
use bevy_openxr::DefaultXrPlugins;
use openxr::{HandJoint, Posef};

fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins)
        //.add_plugins(OpenXrDebugRenderer) //new debug renderer adds gizmos to
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, proto_locomotion)
<<<<<<< HEAD
=======
        .add_systems(Startup, spawn_controllers_example)
        .add_systems(Update, draw_skeleton_hands)
<<<<<<< HEAD
>>>>>>> 68cdf19 (both hands work)
        .insert_resource(PrototypeLocomotionConfig::default())
        .add_systems(Startup, spawn_controllers_example)
        .add_plugins(OpenXrHandInput)
        .add_plugins(HandInputDebugRenderer)
=======
        .add_systems(PreUpdate, update_hand_states)
        .add_systems(PostUpdate, draw_hand_entities)
        .add_systems(Startup, spawn_hand_entities)
        .insert_resource(PrototypeLocomotionConfig::default())
        .insert_resource(HandStatesResource::default())
>>>>>>> 319a2dc (hand state resource is used to drive skeleton)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(Color::rgb(0.8, 0.0, 0.0).into()),
        transform: Transform::from_xyz(0.0, 0.5, 1.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.25, 1.25, 0.0).looking_at(
            Vec3 {
                x: -0.548,
                y: -0.161,
                z: -0.137,
            },
            Vec3::Y,
        ),
        ..default()
    },));
}

pub fn spawn_hand_entities(mut commands: Commands) {
    let hands = [Hand::Left, Hand::Right];
    let bones = [
        HandBone::Palm,
        HandBone::Wrist,
        HandBone::ThumbMetacarpal,
        HandBone::ThumbProximal,
        HandBone::ThumbDistal,
        HandBone::ThumbTip,
        HandBone::IndexMetacarpal,
        HandBone::IndexProximal,
        HandBone::IndexIntermediate,
        HandBone::IndexDistal,
        HandBone::IndexTip,
        HandBone::MiddleMetacarpal,
        HandBone::MiddleProximal,
        HandBone::MiddleIntermediate,
        HandBone::MiddleDistal,
        HandBone::MiddleTip,
        HandBone::RingMetacarpal,
        HandBone::RingProximal,
        HandBone::RingIntermediate,
        HandBone::RingDistal,
        HandBone::RingTip,
        HandBone::LittleMetacarpal,
        HandBone::LittleProximal,
        HandBone::LittleIntermediate,
        HandBone::LittleDistal,
        HandBone::LittleTip,
    ];

    for hand in hands.iter() {
        for bone in bones.iter() {
            commands.spawn((
                SpatialBundle::default(),
                bone.clone(),
                OpenXRTracker,
                hand.clone(),
            ));
        }
    }
}

pub fn update_hand_states(
    oculus_controller: Res<OculusController>,
    hand_states_option: Option<ResMut<HandStatesResource>>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
) {
    match hand_states_option {
        Some(mut hands) => {
            //lock frame
            let frame_state = *frame_state.lock().unwrap();
            //get controller
            let controller =
                oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

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

fn update_emulated_hand_skeletons(
    right_controller_query: Query<(&GlobalTransform, With<OpenXRRightController>)>,
    left_controller_query: Query<(&GlobalTransform, With<OpenXRLeftController>)>,
    hand_states_option: Option<ResMut<HandStatesResource>>,
    mut hand_bone_query: Query<(&mut Transform, &HandBone, &Hand)>,
) {
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

#[derive(Component, Debug, Clone, Copy)]
pub enum HandBone {
    Palm,
    Wrist,
    ThumbMetacarpal,
    ThumbProximal,
    ThumbDistal,
    ThumbTip,
    IndexMetacarpal,
    IndexProximal,
    IndexIntermediate,
    IndexDistal,
    IndexTip,
    MiddleMetacarpal,
    MiddleProximal,
    MiddleIntermediate,
    MiddleDistal,
    MiddleTip,
    RingMetacarpal,
    RingProximal,
    RingIntermediate,
    RingDistal,
    RingTip,
    LittleMetacarpal,
    LittleProximal,
    LittleIntermediate,
    LittleDistal,
    LittleTip,
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
    grip: f32,
    trigger_state: TriggerState,
    a_button: ButtonState,
    b_button: ButtonState,
    thumbstick: ThumbstickState,
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

fn draw_hand_entities(mut gizmos: Gizmos, query: Query<(&Transform, &HandBone)>) {
    for (transform, hand_bone) in query.iter() {
        let (radius, color) = get_bone_gizmo_style(hand_bone);
        gizmos.sphere(transform.translation, transform.rotation, radius, color);
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

fn update_hand_bones_emulated(
    controller_transform: Transform,
    hand: Hand,
    hand_state: HandState,
    hand_bone_query: &mut Query<(&mut Transform, &HandBone, &Hand)>,
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
    for (mut transform, handbone, bonehand) in hand_bone_query.iter_mut() {
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

fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
    ));
    //right hand
    commands.spawn((
        OpenXRRightController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
    ));
}