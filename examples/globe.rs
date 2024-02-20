use bevy::diagnostic::LogDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::transform::components::Transform;
use bevy_oxr::graphics::XrAppInfo;
use bevy_oxr::resources::XrViews;
use bevy_oxr::xr_input::hands::common::HandInputDebugRenderer;
use bevy_oxr::xr_input::interactions::{
    InteractionEvent, XRDirectInteractor, XRInteractorState, XRRayInteractor, XRSocketInteractor,
};
use bevy_oxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_oxr::xr_input::trackers::{
    AimPose, OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
    OpenXRTrackingRoot,
};
use bevy_oxr::xr_input::Vec3Conv;
use bevy_oxr::DefaultXrPlugins;
use wgpu::{Extent3d, TextureDimension, TextureFormat};

fn main() {
    color_eyre::install().unwrap();

    App::new()
        .add_plugins(DefaultXrPlugins {
            app_info: XrAppInfo {
                name: "Bevy OXR Globe Example".into(),
            },
            ..default()
        })
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (proto_locomotion, pull_to_ground).chain())
        .insert_resource(PrototypeLocomotionConfig::default())
        .add_systems(Startup, spawn_controllers_example)
        .add_plugins(HandInputDebugRenderer)
        .add_event::<InteractionEvent>()
        .run();
}

#[derive(Component)]
struct Globe {
    radius: f32,
}

/// Creates a colorful test pattern
fn uv_debug_texture() -> Image {
    const TEXTURE_SIZE: usize = 8;

    let mut palette: [u8; 32] = [
        255, 102, 159, 255, 255, 159, 102, 255, 236, 255, 102, 255, 121, 255, 102, 255, 102, 255,
        198, 255, 102, 198, 255, 255, 121, 102, 255, 255, 236, 102, 255, 255,
    ];

    let mut texture_data = [0; TEXTURE_SIZE * TEXTURE_SIZE * 4];
    for y in 0..TEXTURE_SIZE {
        let offset = TEXTURE_SIZE * y * 4;
        texture_data[offset..(offset + TEXTURE_SIZE * 4)].copy_from_slice(&palette);
        palette.rotate_right(4);
    }

    Image::new_fill(
        Extent3d {
            width: TEXTURE_SIZE as u32,
            height: TEXTURE_SIZE as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &texture_data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // plane
    let radius = 5.0;
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(
                Sphere::new(radius)
            ),
            material: materials.add(StandardMaterial {
                base_color_texture: Some(images.add(uv_debug_texture())),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, -radius, 0.0),
            ..default()
        },
        Globe { radius },
    ));
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::splat(0.1)).mesh()),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.7, 0.6))),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // socket
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 0.5, 1.0),
            ..default()
        },
        XRInteractorState::Selecting,
        XRSocketInteractor,
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

fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        XRRayInteractor,
        AimPose(Transform::default()),
        XRInteractorState::default(),
    ));
    //right hand
    commands.spawn((
        OpenXRRightController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        XRDirectInteractor,
        XRInteractorState::default(),
    ));
}

fn pull_to_ground(
    time: Res<Time>,
    mut tracking_root_query: Query<&mut Transform, (With<OpenXRTrackingRoot>, Without<Globe>)>,
    globe: Query<(&Transform, &Globe), Without<OpenXRTrackingRoot>>,
    views: ResMut<XrViews>,
) {
    let mut root = tracking_root_query.single_mut();
    let (globe_pos, globe) = globe.single();

    // Get player position (position of playground + position within playground)
    let Some(view) = views.first() else { return };
    let mut hmd_translation = view.pose.position.to_vec3();
    hmd_translation.y = 0.0;
    let local = root.translation;
    let offset = root.rotation.mul_vec3(hmd_translation);
    let global = offset + local;

    let adjustment_rate = (time.delta_seconds() * 10.0).min(1.0);

    // Lower player onto sphere
    let up = (global - globe_pos.translation).normalize();
    let diff = up * globe.radius + globe_pos.translation - offset - root.translation;
    root.translation += diff * adjustment_rate;

    // Rotate player to be upright on sphere
    let angle_diff = Quat::from_rotation_arc(*root.up(), up);
    let point = root.translation + offset;
    root.rotate_around(point, Quat::IDENTITY.slerp(angle_diff, adjustment_rate));
}
