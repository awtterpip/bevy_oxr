use std::f32::consts::PI;
use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::Gizmos;
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession, XrViews};
use bevy_openxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_openxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
    OpenXRTrackingRoot, adopt_open_xr_trackers,
};
use bevy_openxr::xr_input::{Hand, QuatConv, Vec3Conv};
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
        .add_systems(Startup, spawn_controllers_example)
        .add_systems(Update, update_open_xr_controllers)
        .add_systems(Update, adopt_open_xr_trackers)
        .insert_resource(PrototypeLocomotionConfig::default())
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

fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        // PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        //     material: materials.add(Color::RED.into()),
        //     transform: Transform::from_xyz(0.0, 0.5, 1.0),
        //     ..default()
        // },
    ));
    //right hand
    commands.spawn((
        OpenXRRightController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        // PbrBundle {
        //     mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        //     material: materials.add(Color::BLUE.into()),
        //     transform: Transform::from_xyz(0.0, 0.5, 1.0),
        //     ..default()
        // },
    ));
}

fn update_open_xr_controllers(
    oculus_controller: Res<OculusController>,
    mut left_controller_query: Query<(
        &mut Transform,
        With<OpenXRLeftController>,
        Without<OpenXRRightController>,
    )>,
    mut right_controller_query: Query<(
        &mut Transform,
        With<OpenXRRightController>,
        Without<OpenXRLeftController>,
    )>,
    frame_state: Res<XrFrameState>,
    instance: Res<XrInstance>,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
) {
    //lock dat frame?
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);
    //get left controller
    let left = controller.grip_space(Hand::Left);
    let left_postion = left.0.pose.position.to_vec3();

    left_controller_query
        .get_single_mut()
        .unwrap()
        .0
        .translation = left_postion;

    left_controller_query.get_single_mut().unwrap().0.rotation = left.0.pose.orientation.to_quat();
    //get right controller
    let right = controller.grip_space(Hand::Right);
    let right_postion = right.0.pose.position.to_vec3();

    right_controller_query
        .get_single_mut()
        .unwrap()
        .0
        .translation = right_postion;

    right_controller_query.get_single_mut().unwrap().0.rotation =
        right.0.pose.orientation.to_quat();
}


