use bevy::{
    prelude::{
        shape, Assets, Camera3dBundle, Color, Commands, Mesh, PbrBundle, PointLight,
        PointLightBundle, ResMut, SpatialBundle, StandardMaterial, Transform, Vec3,
    },
    transform::TransformBundle,
    utils::default,
};
use bevy_oxr::xr_input::interactions::{Touched, XRInteractable, XRInteractableState};
use bevy_rapier3d::{
    prelude::{Collider, RigidBody, Group, CollisionGroups},
    render::ColliderDebugColor,
};

use crate::Grabbable;

/// set up a simple 3D scene
pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    /*
     * workbench plane
     */
    let ground_size = 2.5;
    let ground_height = 0.825;
    let ground_thickness = 0.05;
    // plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(5.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(0.0, ground_height, 0.0),
            ..default()
        },
        RigidBody::Fixed,
        Collider::cuboid(ground_size, ground_thickness, ground_size),
        CollisionGroups::new(Group::GROUP_3, Group::ALL),
    ));
    // cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(0.05, 0.05, 0.05),
        ColliderDebugColor(Color::hsl(220.0, 1.0, 0.3)),
        XRInteractable,
        XRInteractableState::default(),
        Grabbable,
        Touched(false),
    ));

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
