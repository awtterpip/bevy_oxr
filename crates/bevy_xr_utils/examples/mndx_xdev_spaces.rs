use bevy::prelude::*;
use bevy_mod_openxr::{add_xr_plugins, exts::OxrExtensions, init::OxrInitPlugin, resources::OxrSessionConfig};
use bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin;
use bevy_xr_utils::{
    generic_tracker::GenericTrackerGizmoPlugin, mndx_xdev_spaces_trackers::MonadoXDevSpacesPlugin,
};
use openxr::EnvironmentBlendMode;
fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).set(OxrInitPlugin {
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_hand_tracking();
                exts.other.push(c"XR_MNDX_xdev_space".to_bytes_with_nul().to_vec());
                exts
            },
            ..Default::default()
        }))
        .insert_resource(OxrSessionConfig {
            blend_mode_preference: vec![
                EnvironmentBlendMode::ALPHA_BLEND,
                EnvironmentBlendMode::ADDITIVE,
                EnvironmentBlendMode::OPAQUE,
            ],
            ..default()
        })
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins((
            HandGizmosPlugin,
            GenericTrackerGizmoPlugin,
            MonadoXDevSpacesPlugin,
        ))
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
