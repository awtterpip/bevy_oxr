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
    mut config: ResMut<PrototypeLocomotionConfig>,
    action_sets: Res<XrActionSets>,
) {
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
    let mut position = tracking_root_query
        .get_single_mut()
        .expect("too many tracking roots");
    let Some(view) = views.first() else {
        info!("locomotion found no head to use for relative movement");
        return;
    };
    // Get the stick input
    let stick = controller.thumbstick(Hand::Left);
    let input = stick.x * *position.right() + stick.y * *position.forward();
    let reference_quat = match config.locomotion_type {
        LocomotionType::Head => view.pose,
        LocomotionType::Hand => {
            let (loc, _vel) = controller.grip_space(Hand::Left);
            loc.pose
        }
    }
    .orientation
    .to_quat();
    // Get rotation around "up" axis (y).
    let (yaw, _pitch, _roll) = reference_quat.to_euler(EulerRot::YXZ);
    // A quat representing the global position and rotation, ignoring pitch and roll
    let reference_quat = Quat::from_axis_angle(*position.up(), yaw);
    // Direction to move towards, in global orientation.
    let locomotion_vec = reference_quat.mul_vec3(input);
    // Actually move the player
    position.translation += locomotion_vec * config.locomotion_speed * time.delta_seconds();

    // Now rotate the player

    // Get controller direction and "turning strength"
    let control_stick = controller.thumbstick(Hand::Right);
    let rot_input = -control_stick.x; //why is this negative i dont know
    if rot_input.abs() <= config.rotation_stick_deadzone {
        return;
    }
    let angle = match config.rotation_type {
        RotationType::Smooth => rot_input * config.smooth_rotation_speed * time.delta_seconds(),
        RotationType::Snap => {
            // The timer is needed so we don't move by the snap amount every frame.
            // FIXME: use no timer at all and require moving the controller stick back to the default
            // location. This is what most VR games do.
            config.rotation_timer.timer.tick(time.delta());
            if config.rotation_timer.timer.finished() {
                config.rotation_timer.timer.reset();
                return;
            } else {
                config.snap_angle * rot_input.signum()
            }
        }
    };

    // Rotate around the body, not the head
    let smoth_rot = Quat::from_axis_angle(*position.up(), angle);
    // Apply rotation
    let mut hmd_translation = view.pose.position.to_vec3();
    hmd_translation.y = 0.0;
    let local = position.translation;
    let global = position.rotation.mul_vec3(hmd_translation) + local;
    gizmos.circle(global, position.up(), 0.1, Color::GREEN);
    position.rotate_around(global, smoth_rot);
}
