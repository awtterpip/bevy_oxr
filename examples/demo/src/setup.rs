use bevy::{prelude::{Commands, ResMut, Assets, Mesh, StandardMaterial, PbrBundle, shape, Color, Transform, SpatialBundle, Camera3dBundle, Vec3, PointLight, PointLightBundle}, utils::default};

/// set up a simple 3D scene
pub fn setup_scene(
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