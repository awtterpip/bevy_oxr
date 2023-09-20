use std::f32::consts::PI;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::Gizmos;
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession, XrViews};
use bevy_openxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::{Hand, QuatConv, TrackingRoot, Vec3Conv};
use bevy_openxr::DefaultXrPlugins;

fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins)
        .add_plugins(OpenXrDebugRenderer) //new debug renderer adds gizmos to
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, proto_locomotion)
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
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
}

pub enum LocomotionType {
    Head,
    Hand,
}

pub enum RotationType {
    Smooth,
    Snap,
}

fn proto_locomotion(
    time: Res<Time>,
    mut tracking_root_query: Query<(&mut Transform, With<TrackingRoot>)>,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    views: ResMut<XrViews>,
    mut gizmos: Gizmos,
) {
    //lock frame
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);
    let root = tracking_root_query.get_single_mut();
    match root {
        Ok(mut position) => {
            //get the stick input and do some maths
            let stick = controller.thumbstick(Hand::Left);
            let input = Vec3::new(stick.x, 0.0, -stick.y);
            let speed = 1.0;
            //now the question is how do we do hmd based locomotion
            //or controller based for that matter
            let locomotion_type = LocomotionType::Head;
            let mut reference_quat = Quat::IDENTITY;
            match locomotion_type {
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
                        .mul_quat(position.0.rotation); //TODO add root tracking quat to this so we simulate the global rotation
                }
            }
            let mut locomotion_vec = reference_quat.mul_vec3(input);
            locomotion_vec.y = 0.0;
            position.0.translation += locomotion_vec * speed * time.delta_seconds();

            //now time for rotation
            let snap_angle = 45.0;
            let smooth_rotation_speed = 0.5 * PI;
            let rotation_setting = RotationType::Smooth;
            match rotation_setting {
                RotationType::Smooth => {
                    //once again with the math
                    let control_stick = controller.thumbstick(Hand::Right);
                    let rot_input = -control_stick.x; //why is this negative i dont know
                    if rot_input.abs() <= 0.2 {
                        //return;
                    }
                    let smoth_rot = Quat::from_rotation_y(
                        rot_input * smooth_rotation_speed * time.delta_seconds(),
                    );
                    //apply rotation
                    let v = views.lock().unwrap();
                    let views = v.get(0);
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
                    //yup even more math i cant remember
                    todo!();
                }
            }
        }
        Err(_) => info!("too many tracking roots"),
    }
}
