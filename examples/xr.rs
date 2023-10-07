use std::f32::consts::PI;
use std::ops::Mul;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::vec3;
use bevy::transform::components::Transform;
use bevy::{gizmos, prelude::*};
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession};
use bevy_openxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_openxr::xr_input::hand_poses::*;
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_openxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
};
use bevy_openxr::xr_input::{Hand, QuatConv, Vec3Conv};
use bevy_openxr::DefaultXrPlugins;
use openxr::{HandJoint, Posef, Quaternionf, Vector3f};

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
        .add_systems(Startup, spawn_controllers_example)
        .add_systems(Update, draw_skeleton_hands)
        .add_systems(PreUpdate, update_hand_states)
        .insert_resource(PrototypeLocomotionConfig::default())
        .insert_resource(HandStatesResource::default())
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

fn draw_skeleton_hands(
    mut commands: Commands,
    mut gizmos: Gizmos,
    right_controller_query: Query<(&GlobalTransform, With<OpenXRRightController>)>,
    left_controller_query: Query<(&GlobalTransform, With<OpenXRLeftController>)>,
    hand_states_option: Option<ResMut<HandStatesResource>>,
) {
    match hand_states_option {
        Some(hands) => {
            let left_hand_transform = left_controller_query
                .get_single()
                .unwrap()
                .0
                .compute_transform();
            draw_hand_bones(&mut gizmos, left_hand_transform, Hand::Left, hands.left);
            let right_hand_transform = right_controller_query
                .get_single()
                .unwrap()
                .0
                .compute_transform();
            // draw_hand(&mut gizmos, right_hand_transform, Hand::Right);
            draw_hand_bones(&mut gizmos, right_hand_transform, Hand::Right, hands.right);
        }
        None => info!("hand states resource not initialized yet"),
    }
}

fn pose_array_to_transform_array(hand_pose: [Posef; 26]) -> [Transform; 26] {
    let mut result_array: [Transform; 26] = [Transform::default(); 26];
    for (place, data) in result_array.iter_mut().zip(hand_pose.iter()) {
        *place = Transform {
            translation: data.position.to_vec3(),
            rotation: data.orientation.to_quat(),
            scale: Vec3::splat(1.0),
        }
    }
    return result_array;
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

fn draw_hand_bones(
    mut gizmos: &mut Gizmos,
    controller_transform: Transform,
    hand: Hand,
    hand_state: HandState,
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
    //curl represents how closed the hand is from 0 to 1;
    let grip_curl = hand_state.grip;
    let index_curl = hand_state.get_index_curl();
    let thumb_curl = hand_state.get_thumb_curl();
    //get paml quat
    let y = Quat::from_rotation_y(-90.0 * PI / 180.0);
    let x = Quat::from_rotation_x(-90.0 * PI / 180.0);
    let palm_quat = controller_quat.mul_quat(y).mul_quat(x);
    //draw debug rays
    // gizmos.ray(
    //     hand_translation,
    //     palm_quat.mul_vec3(Vec3::Z * 0.2),
    //     Color::BLUE,
    // );
    // gizmos.ray(
    //     hand_translation,
    //     palm_quat.mul_vec3(Vec3::Y * 0.2),
    //     Color::GREEN,
    // );
    // gizmos.ray(
    //     hand_translation,
    //     palm_quat.mul_vec3(Vec3::X * 0.2),
    //     Color::RED,
    // );
    //get simulated bones
    let hand_transform_array: [Transform; 26] = get_simulated_open_hand_transforms(hand);
    //draw controller-palm bone(should be zero length)
    let palm = hand_transform_array[HandJoint::PALM];
    gizmos.ray(hand_translation, palm.translation, Color::WHITE);
    //draw palm-wrist
    let wrist = hand_transform_array[HandJoint::WRIST];
    gizmos.ray(
        hand_translation + palm.translation,
        palm_quat.mul_vec3(wrist.translation),
        Color::GRAY,
    );

    //thumb
    //better finger drawing?
    let thumb_joints = [
        HandJoint::THUMB_METACARPAL,
        HandJoint::THUMB_PROXIMAL,
        HandJoint::THUMB_DISTAL,
        HandJoint::THUMB_TIP,
    ];
    let mut prior_start: Option<Vec3> = None;
    let mut prior_quat: Option<Quat> = None;
    let mut prior_vector: Option<Vec3> = None;
    let color = Color::RED;
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
                gizmos.ray(tp_start, tp_vector, color);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                gizmos.ray(tm_start, tm_vector, color);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
            }
        }
    }

    //index
    //better finger drawing?
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
    let color = Color::ORANGE;
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
                gizmos.ray(tp_start, tp_vector, color);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                gizmos.ray(tm_start, tm_vector, color);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
            }
        }
    }

    //better finger drawing?
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
    let color = Color::YELLOW;
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
                gizmos.ray(tp_start, tp_vector, color);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                gizmos.ray(tm_start, tm_vector, color);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
            }
        }
    }
    //better finger drawing?
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
    let color = Color::GREEN;
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
                gizmos.ray(tp_start, tp_vector, color);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                gizmos.ray(tm_start, tm_vector, color);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
            }
        }
    }

    //better finger drawing?
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
    let color = Color::BLUE;
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
                gizmos.ray(tp_start, tp_vector, color);
                prior_start = Some(tp_start);
                prior_quat = Some(tp_quat);
                prior_vector = Some(tp_vector);
            }
            None => {
                let thumb_meta = hand_transform_array[*bone];
                let tm_start = hand_translation
                    + palm_quat.mul_vec3(palm.translation)
                    + palm_quat.mul_vec3(wrist.translation);
                let tm_vector = palm_quat.mul_vec3(thumb_meta.translation);
                gizmos.ray(tm_start, tm_vector, color);
                prior_start = Some(tm_start);
                prior_quat = Some(splay_quat);
                prior_vector = Some(tm_vector);
            }
        }
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

fn draw_hand(mut gizmos: &mut Gizmos, controller_transform: Transform, hand: Hand) {
    //draw debug for controller grip center to match palm to
    let hand_translation = controller_transform.translation;
    let hand_quat = controller_transform.rotation;
    //we need to flip this i dont know why
    let flip = Quat::from_rotation_x(PI);
    let controller_backward = hand_quat.mul_quat(flip);

    let test_hand_pose = get_test_hand_pose_array();

    let hand_pose = flip_hand_pose(test_hand_pose.clone(), hand);

    // let hand_transform_array: [Transform; 26] = pose_array_to_transform_array(hand_pose);
    let hand_transform_array: [Transform; 26] = get_simulated_open_hand_transforms(hand);

    let palm = hand_transform_array[HandJoint::PALM];
    gizmos.sphere(
        palm.translation + hand_translation,
        palm.rotation.mul_quat(controller_backward),
        0.01,
        Color::WHITE,
    );

    let wrist = hand_transform_array[HandJoint::WRIST];
    draw_joint(
        &mut gizmos,
        wrist.translation,
        wrist.rotation,
        0.01,
        Color::GRAY,
        controller_backward,
        hand_translation,
    );

    let thumb_meta = hand_transform_array[HandJoint::THUMB_METACARPAL];
    draw_joint(
        &mut gizmos,
        thumb_meta.translation,
        thumb_meta.rotation,
        0.01,
        Color::RED,
        controller_backward,
        hand_translation,
    );

    let thumb_prox = hand_transform_array[HandJoint::THUMB_PROXIMAL];
    draw_joint(
        &mut gizmos,
        thumb_prox.translation,
        thumb_prox.rotation,
        0.008,
        Color::RED,
        controller_backward,
        hand_translation,
    );
    let thumb_dist = hand_transform_array[HandJoint::THUMB_DISTAL];
    draw_joint(
        &mut gizmos,
        thumb_dist.translation,
        thumb_dist.rotation,
        0.006,
        Color::RED,
        controller_backward,
        hand_translation,
    );
    let thumb_tip = hand_transform_array[HandJoint::THUMB_TIP];
    draw_joint(
        &mut gizmos,
        thumb_tip.translation,
        thumb_tip.rotation,
        0.004,
        Color::RED,
        controller_backward,
        hand_translation,
    );

    let index_meta = hand_transform_array[HandJoint::INDEX_METACARPAL];
    draw_joint(
        &mut gizmos,
        index_meta.translation,
        index_meta.rotation,
        0.01,
        Color::ORANGE,
        controller_backward,
        hand_translation,
    );
    let index_prox = hand_transform_array[HandJoint::INDEX_PROXIMAL];
    draw_joint(
        &mut gizmos,
        index_prox.translation,
        index_prox.rotation,
        0.008,
        Color::ORANGE,
        controller_backward,
        hand_translation,
    );

    let index_inter = hand_transform_array[HandJoint::INDEX_INTERMEDIATE];
    draw_joint(
        &mut gizmos,
        index_inter.translation,
        index_inter.rotation,
        0.006,
        Color::ORANGE,
        controller_backward,
        hand_translation,
    );

    let index_dist = hand_transform_array[HandJoint::INDEX_DISTAL];
    draw_joint(
        &mut gizmos,
        index_dist.translation,
        index_dist.rotation,
        0.004,
        Color::ORANGE,
        controller_backward,
        hand_translation,
    );

    let index_tip = hand_transform_array[HandJoint::INDEX_TIP];
    draw_joint(
        &mut gizmos,
        index_tip.translation,
        index_tip.rotation,
        0.002,
        Color::ORANGE,
        controller_backward,
        hand_translation,
    );

    let middle_meta = hand_transform_array[HandJoint::MIDDLE_METACARPAL];
    draw_joint(
        &mut gizmos,
        middle_meta.translation,
        middle_meta.rotation,
        0.01,
        Color::YELLOW,
        controller_backward,
        hand_translation,
    );
    let middle_prox = hand_transform_array[HandJoint::MIDDLE_PROXIMAL];
    draw_joint(
        &mut gizmos,
        middle_prox.translation,
        middle_prox.rotation,
        0.008,
        Color::YELLOW,
        controller_backward,
        hand_translation,
    );
    let middle_inter = hand_transform_array[HandJoint::MIDDLE_INTERMEDIATE];
    draw_joint(
        &mut gizmos,
        middle_inter.translation,
        middle_inter.rotation,
        0.006,
        Color::YELLOW,
        controller_backward,
        hand_translation,
    );
    let middle_dist = hand_transform_array[HandJoint::MIDDLE_DISTAL];
    draw_joint(
        &mut gizmos,
        middle_dist.translation,
        middle_dist.rotation,
        0.004,
        Color::YELLOW,
        controller_backward,
        hand_translation,
    );
    let middle_tip = hand_transform_array[HandJoint::MIDDLE_TIP];
    draw_joint(
        &mut gizmos,
        middle_tip.translation,
        middle_tip.rotation,
        0.002,
        Color::YELLOW,
        controller_backward,
        hand_translation,
    );

    let ring_meta = hand_transform_array[HandJoint::RING_METACARPAL];
    draw_joint(
        &mut gizmos,
        ring_meta.translation,
        ring_meta.rotation,
        0.01,
        Color::GREEN,
        controller_backward,
        hand_translation,
    );
    let ring_prox = hand_transform_array[HandJoint::RING_PROXIMAL];
    draw_joint(
        &mut gizmos,
        ring_prox.translation,
        ring_prox.rotation,
        0.008,
        Color::GREEN,
        controller_backward,
        hand_translation,
    );
    let ring_inter = hand_transform_array[HandJoint::RING_INTERMEDIATE];
    draw_joint(
        &mut gizmos,
        ring_inter.translation,
        ring_inter.rotation,
        0.006,
        Color::GREEN,
        controller_backward,
        hand_translation,
    );
    let ring_dist = hand_transform_array[HandJoint::RING_DISTAL];
    draw_joint(
        &mut gizmos,
        ring_dist.translation,
        ring_dist.rotation,
        0.004,
        Color::GREEN,
        controller_backward,
        hand_translation,
    );
    let ring_tip = hand_transform_array[HandJoint::RING_TIP];
    draw_joint(
        &mut gizmos,
        ring_tip.translation,
        ring_tip.rotation,
        0.002,
        Color::GREEN,
        controller_backward,
        hand_translation,
    );

    let little_meta = hand_transform_array[HandJoint::LITTLE_METACARPAL];
    draw_joint(
        &mut gizmos,
        little_meta.translation,
        little_meta.rotation,
        0.01,
        Color::BLUE,
        controller_backward,
        hand_translation,
    );
    let little_prox = hand_transform_array[HandJoint::LITTLE_PROXIMAL];
    draw_joint(
        &mut gizmos,
        little_prox.translation,
        little_prox.rotation,
        0.008,
        Color::BLUE,
        controller_backward,
        hand_translation,
    );
    let little_inter = hand_transform_array[HandJoint::LITTLE_INTERMEDIATE];
    draw_joint(
        &mut gizmos,
        little_inter.translation,
        little_inter.rotation,
        0.006,
        Color::BLUE,
        controller_backward,
        hand_translation,
    );
    let little_dist = hand_transform_array[HandJoint::LITTLE_DISTAL];
    draw_joint(
        &mut gizmos,
        little_dist.translation,
        little_dist.rotation,
        0.004,
        Color::BLUE,
        controller_backward,
        hand_translation,
    );
    let little_tip = hand_transform_array[HandJoint::LITTLE_TIP];
    draw_joint(
        &mut gizmos,
        little_tip.translation,
        little_tip.rotation,
        0.002,
        Color::BLUE,
        controller_backward,
        hand_translation,
    );
}

fn flip_hand_pose(hand_pose: [Posef; 26], hand: Hand) -> [Posef; 26] {
    let mut new_pose = hand_pose;
    match hand {
        Hand::Left => {
            for pose in new_pose.iter_mut() {
                pose.position.x = -pose.position.x;
            }
        }
        Hand::Right => (),
    }
    return new_pose;
}

fn draw_joint(
    gizmos: &mut Gizmos,
    joint_pos: Vec3,
    joint_rot: Quat,
    radius: f32,
    color: Color,
    controller_backwards: Quat,
    offset: Vec3,
) {
    gizmos.sphere(
        controller_backwards.mul_vec3(joint_pos) + offset,
        joint_rot,
        radius,
        color,
    );
}

fn log_hand(hand_pose: [Posef; 26]) {
    let palm_wrist = hand_pose[HandJoint::WRIST].position.to_vec3()
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
