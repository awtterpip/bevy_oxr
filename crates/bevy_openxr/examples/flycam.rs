use bevy::prelude::*;
use bevy_openxr::add_xr_plugins;
use bevy_xr_utils::{
    flycam_locomotion::{FlyCamConfig, FlycamLocomotionPlugin},
    transform_utils,
    xr_utils_actions::XRUtilsActionsPlugin,
};

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_plugins(transform_utils::TransformUtilitiesPlugin)
        .add_systems(Startup, setup)
        .add_plugins(XRUtilsActionsPlugin)
        //flycam specific stuff
        .add_plugins(FlycamLocomotionPlugin)
        .insert_resource(AmbientLight {
            color: Default::default(),
            brightness: 500.0,
        })
        .insert_resource(FlyCamConfig::default())
        .run();
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
    // red cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(252, 44, 3)),
        transform: Transform::from_xyz(4.0, 0.5, 0.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // blue cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(3, 28, 252)),
        transform: Transform::from_xyz(-4.0, 0.5, 0.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // green cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(3, 252, 32)),
        transform: Transform::from_xyz(0.0, 0.5, 4.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // white cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(250, 250, 250)),
        transform: Transform::from_xyz(0.0, 0.5, -4.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // black cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(0, 0, 0)),
        transform: Transform::from_xyz(0.0, 0.1, 0.0).with_scale(Vec3::splat(0.2)),
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
