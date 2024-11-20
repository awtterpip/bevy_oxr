//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_mod_openxr::add_xr_plugins;
use bevy_mod_xr::session::XrSessionCreated;
use bevy_xr_utils::tracking_utils::{
    TrackingUtilitiesPlugin, XrTrackedLeftGrip, XrTrackedLocalFloor, XrTrackedRightGrip,
    XrTrackedStage, XrTrackedView,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins));
    app.add_systems(Startup, setup);

    //things?
    app.add_systems(XrSessionCreated, spawn_hands);

    //tracking utils plugin
    app.add_plugins(TrackingUtilitiesPlugin);

    app.run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_hands(
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let left = cmds
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
            MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
            Transform::from_xyz(0.0, 0.5, 0.0),
            XrTrackedLeftGrip,
        ))
        .id();
    let bundle = (
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        XrTrackedRightGrip,
    );
    let right = cmds.spawn(bundle).id();
    //head

    let head = cmds
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.2, 0.2))),
            MeshMaterial3d(materials.add(Color::srgb_u8(255, 144, 144))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            XrTrackedView,
        ))
        .id();
    //local_floor emulated
    let local_floor = cmds
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.1, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb_u8(144, 255, 144))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            XrTrackedLocalFloor,
        ))
        .id();

    let stage = cmds
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.5, 0.1, 0.5))),
            MeshMaterial3d(materials.add(Color::srgb_u8(144, 255, 255))),
            Transform::from_xyz(0.0, 0.0, 0.0),
            XrTrackedStage,
        ))
        .id();

    cmds.entity(stage)
        .add_children(&[left, right, head, local_floor]);
}
