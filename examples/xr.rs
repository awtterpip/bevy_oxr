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

use bevy_openxr::xr_input::hand::HandsResource;
use bevy_openxr::xr_input::hand_poses::*;
use bevy_openxr::xr_input::oculus_touch::OculusController;
>>>>>>> 319a2dc (hand state resource is used to drive skeleton)
use bevy_openxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_openxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
};
use bevy_openxr::DefaultXrPlugins;

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