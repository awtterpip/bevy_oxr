//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{prelude::*, render::pipelined_rendering::PipelinedRenderingPlugin};
use bevy_mod_openxr::{add_xr_plugins, init::OxrInitPlugin};
use openxr::EnvironmentBlendMode;

fn main() {
    App::new()
        .add_plugins(
            add_xr_plugins(DefaultPlugins.build().disable::<PipelinedRenderingPlugin>()).set(
                OxrInitPlugin {
                    blend_modes: Some(vec![
                        EnvironmentBlendMode::ALPHA_BLEND,
                        EnvironmentBlendMode::ADDITIVE,
                    ]),
                    ..Default::default()
                },
            ),
        )
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup)
        .insert_resource(ClearColor(Color::NONE))
        .run();
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
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
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
