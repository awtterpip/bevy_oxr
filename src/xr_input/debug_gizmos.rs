use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::log::{debug, info};
use bevy::prelude::{
    Color, Gizmos, GlobalTransform, Plugin, Quat, Query, Res, Transform, Update, Vec2, Vec3, With,
    Without,
};

use crate::xr_init::xr_only;
use crate::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
};

use crate::xr_input::{
    oculus_touch::{OculusController, OculusControllerRef},
    Hand,
};

use super::{
    actions::XrActionSets,
    trackers::{OpenXRLeftController, OpenXRRightController, OpenXRTrackingRoot},
};

/// add debug renderer for controllers
#[derive(Default)]
pub struct OpenXrDebugRenderer;

impl Plugin for OpenXrDebugRenderer {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, draw_gizmos.run_if(xr_only()));
    }
}

#[allow(clippy::too_many_arguments, clippy::complexity)]
pub fn draw_gizmos(
    mut gizmos: Gizmos,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
    tracking_root_query: Query<
        &mut Transform,
        (
            With<OpenXRTrackingRoot>,
            Without<OpenXRLeftController>,
            Without<OpenXRRightController>,
        ),
    >,
    left_controller_query: Query<
        &GlobalTransform,
        (
            With<OpenXRLeftController>,
            Without<OpenXRRightController>,
            Without<OpenXRTrackingRoot>,
        ),
    >,
    right_controller_query: Query<
        &GlobalTransform,
        (
            With<OpenXRRightController>,
            Without<OpenXRLeftController>,
            Without<OpenXRTrackingRoot>,
        ),
    >,
    action_sets: Res<XrActionSets>,
) {
    // if let Some(hand_tracking) = hand_tracking {
    //     let handtracking_ref = hand_tracking.get_ref(&xr_input, &frame_state);
    //     if let Some(joints) = handtracking_ref.get_poses(Hand::Left) {
    //         for joint in joints.inner() {
    //             let trans = Transform::from_rotation(joint.orientation);
    //             gizmos.circle(
    //                 joint.position,
    //                 trans.forward(),
    //                 joint.radius,
    //                 Color::ORANGE_RED,
    //             );
    //         }
    //     }
    //     if let Some(joints) = handtracking_ref.get_poses(Hand::Right) {
    //         for joint in joints.inner() {
    //             let trans = Transform::from_rotation(joint.orientation);
    //             gizmos.circle(
    //                 joint.position,
    //                 trans.forward(),
    //                 joint.radius,
    //                 Color::LIME_GREEN,
    //             );
    //         }
    //         return;
    //     }
    // }
    //lock frame
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
    let root = tracking_root_query.get_single();
    match root {
        Ok(position) => {
            gizmos.circle(
                position.translation
                    + Vec3 {
                        x: 0.0,
                        y: 0.01,
                        z: 0.0,
                    },
                Vec3::Y,
                0.2,
                Color::RED,
            );
        }
        Err(_) => info!("too many tracking roots"),
    }
    //draw the hands
    //left
    let left_transform = left_controller_query.get_single();
    match left_transform {
        Ok(left_entity) => {
            draw_hand_gizmo(&mut gizmos, &controller, Hand::Left, left_entity);
        }
        Err(_) => debug!("no left controller entity for debug gizmos"),
    }
    //right
    let right_transform = right_controller_query.get_single();
    match right_transform {
        Ok(right_entity) => {
            draw_hand_gizmo(&mut gizmos, &controller, Hand::Right, right_entity);
        }
        Err(_) => debug!("no right controller entity for debug gizmos"),
    }
}

fn draw_hand_gizmo(
    gizmos: &mut Gizmos,
    controller: &OculusControllerRef<'_>,
    hand: Hand,
    hand_transform: &GlobalTransform,
) {
    match hand {
        Hand::Left => {
            let left_color = Color::YELLOW_GREEN;
            let off_color = Color::BLUE;
            let touch_color = Color::GREEN;
            let pressed_color = Color::RED;

            let grip_quat_offset = Quat::from_rotation_x(-1.4);
            let face_quat_offset = Quat::from_rotation_x(1.05);
            let trans = hand_transform.compute_transform();
            let controller_vec3 = trans.translation;
            let controller_quat = trans.rotation;
            let face_quat = controller_quat.mul_quat(face_quat_offset);
            let face_quat_normal = face_quat.mul_vec3(Vec3::Z);

            //draw grip
            gizmos.rect(
                controller_vec3,
                controller_quat * grip_quat_offset,
                Vec2::new(0.05, 0.1),
                left_color,
            );

            let face_translation_offset = Quat::from_rotation_x(-1.7); //direction to move the face from the controller tracking point
            let face_translation_vec3 = controller_vec3
                + controller_quat
                    .mul_quat(face_translation_offset)
                    .mul_vec3(Vec3::Y * 0.075); //distance to move face by

            //draw face
            gizmos.circle(
                face_translation_vec3,
                face_quat_normal,
                0.04,
                Color::YELLOW_GREEN,
            );

            //button b
            let mut b_color = off_color;
            if controller.y_button_touched() {
                b_color = touch_color;
            }
            if controller.y_button() {
                b_color = pressed_color;
            }

            let b_offset_quat = face_quat;
            let b_translation_vec3 =
                face_translation_vec3 + b_offset_quat.mul_vec3(Vec3::new(0.025, -0.01, 0.0));
            gizmos.circle(b_translation_vec3, face_quat_normal, 0.0075, b_color);

            //button a
            let mut a_color = off_color;
            if controller.x_button_touched() {
                a_color = touch_color;
            }
            if controller.x_button() {
                a_color = pressed_color;
            }

            let a_offset_quat = face_quat;
            let a_translation_vec3 =
                face_translation_vec3 + a_offset_quat.mul_vec3(Vec3::new(0.025, 0.01, 0.0));
            gizmos.circle(a_translation_vec3, face_quat_normal, 0.0075, a_color);

            //joystick
            let joystick_offset_quat = face_quat;
            let joystick_base_vec =
                face_translation_vec3 + joystick_offset_quat.mul_vec3(Vec3::new(-0.02, 0.0, 0.0));
            let mut joystick_color = off_color;
            if controller.thumbstick_touch(Hand::Left) {
                joystick_color = touch_color;
            }

            //base
            gizmos.circle(joystick_base_vec, face_quat_normal, 0.014, joystick_color);

            let stick = controller.thumbstick(Hand::Left);
            let input = Vec3::new(stick.x, -stick.y, 0.0);
            let joystick_top_vec = face_translation_vec3
                + joystick_offset_quat.mul_vec3(Vec3::new(-0.02, 0.0, -0.01))
                + joystick_offset_quat.mul_vec3(input * 0.01);
            //top
            gizmos.circle(joystick_top_vec, face_quat_normal, 0.005, joystick_color);

            //trigger
            let trigger_state = controller.trigger(Hand::Left);
            let trigger_rotation = Quat::from_rotation_x(-0.75 * trigger_state);
            let mut trigger_color = off_color;
            if controller.trigger_touched(Hand::Left) {
                trigger_color = touch_color;
            }
            let trigger_transform = Transform {
                translation: face_translation_vec3
                    + face_quat
                        .mul_quat(trigger_rotation)
                        .mul_vec3(Vec3::new(0.0, 0.0, 0.02)),
                rotation: face_quat.mul_quat(trigger_rotation),
                scale: Vec3 {
                    x: 0.01,
                    y: 0.02,
                    z: 0.03,
                },
            };
            gizmos.cuboid(trigger_transform, trigger_color);
        }
        Hand::Right => {
            //get right controller
            let right_color = Color::YELLOW_GREEN;
            let off_color = Color::BLUE;
            let touch_color = Color::GREEN;
            let pressed_color = Color::RED;

            let grip_quat_offset = Quat::from_rotation_x(-1.4);
            let face_quat_offset = Quat::from_rotation_x(1.05);

            let trans = hand_transform.compute_transform();
            let controller_vec3 = trans.translation;
            let controller_quat = trans.rotation;
            let face_quat = controller_quat.mul_quat(face_quat_offset);
            let face_quat_normal = face_quat.mul_vec3(Vec3::Z);

            let _squeeze = controller.squeeze(Hand::Right);
            //info!("{:?}", squeeze);
            //grip
            gizmos.rect(
                controller_vec3,
                controller_quat * grip_quat_offset,
                Vec2::new(0.05, 0.1),
                right_color,
            );

            let face_translation_offset = Quat::from_rotation_x(-1.7); //direction to move the face from the controller tracking point
            let face_translation_vec3 = controller_vec3
                + controller_quat
                    .mul_quat(face_translation_offset)
                    .mul_vec3(Vec3::Y * 0.075); //distance to move face by

            //draw face
            gizmos.circle(
                face_translation_vec3,
                face_quat_normal,
                0.04,
                Color::YELLOW_GREEN,
            );

            //button b
            let mut b_color = off_color;
            if controller.b_button_touched() {
                b_color = touch_color;
            }
            if controller.b_button() {
                b_color = pressed_color;
            }

            let b_offset_quat = face_quat;
            let b_translation_vec3 =
                face_translation_vec3 + b_offset_quat.mul_vec3(Vec3::new(-0.025, -0.01, 0.0));
            gizmos.circle(b_translation_vec3, face_quat_normal, 0.0075, b_color);

            //button a
            let mut a_color = off_color;
            if controller.a_button_touched() {
                a_color = touch_color;
            }
            if controller.a_button() {
                a_color = pressed_color;
            }

            let a_offset_quat = face_quat;
            let a_translation_vec3 =
                face_translation_vec3 + a_offset_quat.mul_vec3(Vec3::new(-0.025, 0.01, 0.0));
            gizmos.circle(a_translation_vec3, face_quat_normal, 0.0075, a_color);

            //joystick time
            let joystick_offset_quat = face_quat;
            let joystick_base_vec =
                face_translation_vec3 + joystick_offset_quat.mul_vec3(Vec3::new(0.02, 0.0, 0.0));
            let mut joystick_color = off_color;
            if controller.thumbstick_touch(Hand::Right) {
                joystick_color = touch_color;
            }

            //base
            gizmos.circle(joystick_base_vec, face_quat_normal, 0.014, joystick_color);

            let stick = controller.thumbstick(Hand::Right);
            let input = Vec3::new(stick.x, -stick.y, 0.0);
            let joystick_top_vec = face_translation_vec3
                + joystick_offset_quat.mul_vec3(Vec3::new(0.02, 0.0, -0.01))
                + joystick_offset_quat.mul_vec3(input * 0.01);
            //top
            gizmos.circle(joystick_top_vec, face_quat_normal, 0.005, joystick_color);

            //trigger
            let trigger_state = controller.trigger(Hand::Right);
            let trigger_rotation = Quat::from_rotation_x(-0.75 * trigger_state);
            let mut trigger_color = off_color;
            if controller.trigger_touched(Hand::Right) {
                trigger_color = touch_color;
            }
            let trigger_transform = Transform {
                translation: face_translation_vec3
                    + face_quat
                        .mul_quat(trigger_rotation)
                        .mul_vec3(Vec3::new(0.0, 0.0, 0.02)),
                rotation: face_quat.mul_quat(trigger_rotation),
                scale: Vec3 {
                    x: 0.01,
                    y: 0.02,
                    z: 0.03,
                },
            };
            gizmos.cuboid(trigger_transform, trigger_color);
        }
    }
}
