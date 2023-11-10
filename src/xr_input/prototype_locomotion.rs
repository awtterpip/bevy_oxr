use std::f32::consts::PI;

use bevy::{
    prelude::*,
    time::{Time, Timer, TimerMode},
};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession, XrViews},
};

use super::{
    actions::ActionSets, oculus_touch::OculusController, trackers::OpenXRTrackingRoot, Hand,
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
    mut tracking_root_query: Query<(&mut Transform, With<OpenXRTrackingRoot>)>,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    views: ResMut<XrViews>,
    mut gizmos: Gizmos,
    config_option: Option<ResMut<PrototypeLocomotionConfig>>,
    action_sets: Res<ActionSets>,
) {
    match config_option {
        Some(_) => (),
        None => {
            info!("no locomotion config");
            return;
        }
    }
    //i hate this but im too tired to think
    let mut config = config_option.unwrap();
    //lock frame
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
    let root = tracking_root_query.get_single_mut();
    match root {
        Ok(mut position) => {
            //get the stick input and do some maths
            let stick = controller.thumbstick(Hand::Left);
            let input = Vec3::new(stick.x, 0.0, -stick.y);

            let mut reference_quat = Quat::IDENTITY;
            match config.locomotion_type {
                LocomotionType::Head => {
                    let v = views.lock().unwrap();
                    let views = v.get(0);
                    match views {
                        Some(view) => {
                            reference_quat = view
                                .pose
                                .orientation
                                .to_quat()
                                .mul_quat(position.0.rotation);
                        }
                        None => return,
                    }
                }
                LocomotionType::Hand => {
                    let grip = controller.grip_space(Hand::Left);
                    reference_quat = grip
                        .0
                        .pose
                        .orientation
                        .to_quat()
                        .mul_quat(position.0.rotation);
                }
            }
            //TODO: do this correctly as just removing the y from the resultant vec3 isnt correct, but works well enough for now
            let mut locomotion_vec = reference_quat.mul_vec3(input);
            locomotion_vec.y = 0.0;
            position.0.translation +=
                locomotion_vec * config.locomotion_speed * time.delta_seconds();

            //now time for rotation

            match config.rotation_type {
                RotationType::Smooth => {
                    //once again with the math
                    let control_stick = controller.thumbstick(Hand::Right);
                    let rot_input = -control_stick.x; //why is this negative i dont know
                    if rot_input.abs() <= config.rotation_stick_deadzone {
                        return;
                    }
                    let smoth_rot = Quat::from_rotation_y(
                        rot_input * config.smooth_rotation_speed * time.delta_seconds(),
                    );
                    //apply rotation
                    let v = views.lock().unwrap();
                    let views = v.get(0);
                    match views {
                        Some(view) => {
                            let mut hmd_translation = view.pose.position.to_vec3();
                            hmd_translation.y = 0.0;
                            let local = position.0.translation;
                            let global = position.0.rotation.mul_vec3(hmd_translation) + local;
                            gizmos.circle(global, Vec3::Y, 0.1, Color::GREEN);
                            position.0.rotate_around(global, smoth_rot);
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
                        let smoth_rot = Quat::from_rotation_y(config.snap_angle * dir);
                        //apply rotation
                        let v = views.lock().unwrap();
                        let views = v.get(0);
                        match views {
                            Some(view) => {
                                let mut hmd_translation = view.pose.position.to_vec3();
                                hmd_translation.y = 0.0;
                                let local = position.0.translation;
                                let global = position.0.rotation.mul_vec3(hmd_translation) + local;
                                gizmos.circle(global, Vec3::Y, 0.1, Color::GREEN);
                                position.0.rotate_around(global, smoth_rot);
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
