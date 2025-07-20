//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{prelude::*, render::pipelined_rendering::PipelinedRenderingPlugin};
use bevy_mod_openxr::{
    add_xr_plugins,
    graphics::{GraphicsBackend, OxrManualGraphicsConfig},
};

fn main() -> AppExit {
    App::new()
        .insert_resource(OxrManualGraphicsConfig {
            fallback_backend: GraphicsBackend::Vulkan(()),
            vk_instance_exts: vec![],
            vk_device_exts: vec![ash::khr::external_memory::NAME],
        })
        .add_plugins(add_xr_plugins(
            // Disabling Pipelined Rendering should reduce latency a little bit for button inputs
            // and increase accuracy for hand tracking, controller positions and similar,
            // the views are updated right before rendering so they are as accurate as possible
            DefaultPlugins.build().disable::<PipelinedRenderingPlugin>(),
        ))
        .add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup)
        .run()
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
