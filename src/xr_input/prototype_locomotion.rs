use std::f32::consts::PI;

use bevy::{
    color::palettes, prelude::*, time::{Time, Timer, TimerMode}
};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession, XrViews},
};

use super::{
    actions::XrActionSets, oculus_touch::OculusController, trackers::OpenXRTrackingRoot, Hand,
    QuatConv, Vec3Conv,
};

pub enum LocomotionType {
    Head,
    Hand,
}

pub enum RotationType {
    Smooth,
    Snap,
}

#[derive(Resource)]
pub struct RotationTimer {
    pub timer: Timer,
}

#[derive(Resource)]
pub struct PrototypeLocomotionConfig {
    pub locomotion_type: LocomotionType,
    pub locomotion_speed: f32,
    pub rotation_type: RotationType,
    pub snap_angle: f32,
    pub smooth_rotation_speed: f32,
    pub rotation_stick_deadzone: f32,
    pub rotation_timer: RotationTimer,
}

impl Default for PrototypeLocomotionConfig {
    fn default() -> Self {
        Self {
            locomotion_type: LocomotionType::Head,
            locomotion_speed: 1.0,
            rotation_type: RotationType::Smooth,
            snap_angle: 45.0 * (PI / 180.0),
            smooth_rotation_speed: 0.5 * PI,
            rotation_stick_deadzone: 0.2,
            rotation_timer: RotationTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            },
        }
    }
}

pub fn proto_locomotion(
    time: Res<Time>,
    mut tracking_root_query: Query<&mut Transform, With<OpenXRTrackingRoot>>,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
    views: ResMut<XrViews>,
    mut gizmos: Gizmos,
    config_option: Option<ResMut<PrototypeLocomotionConfig>>,
    action_sets: Res<XrActionSets>,
) {
    let mut config = match config_option {
        Some(c) => c,
        None => {
            info!("no locomotion config");
            return;
        }
    };
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
    let root = tracking_root_query.get_single_mut();
    match root {
        Ok(mut position) => {
            //get the stick input and do some maths
            let stick = controller.thumbstick(Hand::Left);
            let input = stick.x * *position.right() + stick.y * *position.forward();
            let reference_quat;
            match config.locomotion_type {
                LocomotionType::Head => {
                    let views = views.first();
                    match views {
                        Some(view) => {
                            reference_quat = view.pose.orientation.to_quat();
                        }
                        None => return,
                    }
                }
                LocomotionType::Hand => {
                    let grip = controller.grip_space(Hand::Left);
                    reference_quat = grip.0.pose.orientation.to_quat();
                }
            }
            let (yaw, _pitch, _roll) = reference_quat.to_euler(EulerRot::YXZ);
            let reference_quat = Quat::from_axis_angle(*position.up(), yaw);
            let locomotion_vec = reference_quat.mul_vec3(input);
            position.translation += locomotion_vec * config.locomotion_speed * time.delta_seconds();

            //now time for rotation

            match config.rotation_type {
                RotationType::Smooth => {
                    //once again with the math
                    let control_stick = controller.thumbstick(Hand::Right);
                    let rot_input = -control_stick.x; //why is this negative i dont know
                    if rot_input.abs() <= config.rotation_stick_deadzone {
                        return;
                    }
                    let smoth_rot = Quat::from_axis_angle(
                        *position.up(),
                        rot_input * config.smooth_rotation_speed * time.delta_seconds(),
                    );
                    //apply rotation
                    let views = views.first();
                    match views {
                        Some(view) => {
                            let mut hmd_translation = view.pose.position.to_vec3();
                            hmd_translation.y = 0.0;
                            let local = position.translation;
                            let global = position.rotation.mul_vec3(hmd_translation) + local;
                            gizmos.circle(global, position.up(), 0.1, palettes::css::GREEN);
                            position.rotate_around(global, smoth_rot);
                        }
                        None => return,
                    }
                }
                RotationType::Snap => {
                    //tick the timer
                    config.rotation_timer.timer.tick(time.delta());
                    if config.rotation_timer.timer.finished() {
                        //now we can snap turn?
                        //once again with the math
                        let control_stick = controller.thumbstick(Hand::Right);
                        let rot_input = -control_stick.x;
                        if rot_input.abs() <= config.rotation_stick_deadzone {
                            return;
                        }
                        let dir: f32 = match rot_input > 0.0 {
                            true => 1.0,
                            false => -1.0,
                        };
                        let smoth_rot =
                            Quat::from_axis_angle(*position.up(), config.snap_angle * dir);
                        //apply rotation
                        let v = views;
                        let views = v.first();
                        match views {
                            Some(view) => {
                                let mut hmd_translation = view.pose.position.to_vec3();
                                hmd_translation.y = 0.0;
                                let local = position.translation;
                                let global = position.rotation.mul_vec3(hmd_translation) + local;
                                gizmos.circle(global, position.up(), 0.1, palettes::css::GREEN);
                                position.rotate_around(global, smoth_rot);
                            }
                            None => return,
                        }
                        config.rotation_timer.timer.reset();
                    }
                }
            }
        }
        Err(_) => info!("too many tracking roots"),
    }
}
