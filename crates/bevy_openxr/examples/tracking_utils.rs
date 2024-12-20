//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_mod_openxr::{action_binding::OxrSendActionBindings, add_xr_plugins};
use bevy_mod_xr::session::XrSessionCreated;
use bevy_xr_utils::tracking_utils::{
    suggest_action_bindings, TrackingUtilitiesPlugin, XrTrackedLeftGrip, XrTrackedLocalFloor, XrTrackedRightGrip, XrTrackedStage, XrTrackedView
};

fn main() {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins));
    app.add_systems(Startup, setup);

    //things?
    app.add_systems(XrSessionCreated, spawn_hands);

    //tracking utils plugin
    app.add_plugins(TrackingUtilitiesPlugin);
    //default bindings only use for prototyping
    app.add_systems(OxrSendActionBindings, suggest_action_bindings);

    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn spawn_hands(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let left = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.05)),
                material: materials.add(Color::srgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            XrTrackedLeftGrip,
        ))
        .id();
    let bundle = (
        PbrBundle {
            mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.05)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        XrTrackedRightGrip,
    );
    let right = cmds.spawn(bundle).id();
    //head

    let head = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
                material: materials.add(Color::srgb_u8(255, 144, 144)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            XrTrackedView,
        ))
        .id();
    //local_floor emulated
    let local_floor = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.5, 0.1, 0.5)),
                material: materials.add(Color::srgb_u8(144, 255, 144)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            XrTrackedLocalFloor,
        ))
        .id();

    let stage = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.5, 0.1, 0.5)),
                material: materials.add(Color::srgb_u8(144, 255, 255)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            XrTrackedStage,
        ))
        .id();

    cmds.entity(stage)
        .push_children(&[left, right, head, local_floor]);
}
