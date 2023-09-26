use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_openxr::xr_input::{Vec3Conv, QuatConv};
use bevy_openxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_openxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_openxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
};
use bevy_openxr::DefaultXrPlugins;
use openxr::{Posef, Quaternionf, Vector3f, HandJoint};

fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins)
        .add_plugins(OpenXrDebugRenderer) //new debug renderer adds gizmos to
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, proto_locomotion)
        .add_systems(Startup, spawn_controllers_example)
        .add_systems(Update, spawn_skeleton)
        .insert_resource(PrototypeLocomotionConfig::default())
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
        transform: Transform::from_xyz(0.25, 1.25, 0.0).looking_at(Vec3{x: -0.548, y: -0.161, z: -0.137}, Vec3::Y),
        ..default()
    },));
}

fn spawn_skeleton(mut commands: Commands, mut gizmos: Gizmos) {
    let hand_pose: [Posef; 26] = [
        Posef { position: Vector3f {x: -0.548, y: -0.161, z: -0.137}, orientation: Quaternionf {x: -0.267,y:  0.849, z: 0.204,w:  0.407}}, //palm
        Posef { position: Vector3f {x: -0.548, y: 0.161, z: -0.137}, orientation: Quaternionf {x: -0.267,y:  0.849, z: 0.204,w:  0.407}},
        Posef { position: Vector3f {x: -0.529, y: -0.198, z: -0.126}, orientation: Quaternionf {x: -0.744, y: -0.530, z: 0.156,w:  -0.376}},
        Posef { position: Vector3f {x: -0.533, y: -0.175, z: -0.090}, orientation: Quaternionf {x: -0.786, y: -0.550, z: 0.126,w:  -0.254}},
        Posef { position: Vector3f {x: -0.544, y: -0.158, z: -0.069}, orientation: Quaternionf {x: -0.729, y: -0.564, z: 0.027,w:  -0.387}}, 
        Posef { position: Vector3f {x: -0.557, y: -0.150, z: -0.065}, orientation: Quaternionf {x: -0.585, y: -0.548, z: -0.140,w:  -0.582}},
        Posef { position: Vector3f {x: -0.521, y: -0.182, z: -0.136}, orientation: Quaternionf {x: -0.277, y: -0.826, z: 0.317,w:  -0.376}},
        Posef { position: Vector3f {x: -0.550, y: -0.135, z: -0.102}, orientation: Quaternionf {x: -0.277, y: -0.826, z: 0.317,w:  -0.376}},
        Posef { position: Vector3f {x: -0.571, y: -0.112, z: -0.082}, orientation: Quaternionf {x: -0.244, y: -0.843, z: 0.256,w:  -0.404}},
        Posef { position: Vector3f {x: -0.585, y: -0.102, z: -0.070}, orientation: Quaternionf {x: -0.200, y: -0.866, z: 0.165,w:  -0.428}}, 
        Posef { position: Vector3f {x: -0.593, y: -0.098, z: -0.064}, orientation: Quaternionf {x: -0.172, y: -0.874, z: 0.110,w:  -0.440}},
        Posef { position: Vector3f {x: -0.527, y: -0.178, z: -0.144}, orientation: Quaternionf {x: -0.185, y: -0.817, z: 0.370,w:  -0.401}},
        Posef { position: Vector3f {x: -0.559, y: -0.132, z: -0.119}, orientation: Quaternionf {x: -0.185, y: -0.817, z: 0.370,w:  -0.401}},
        Posef { position: Vector3f {x: -0.582, y: -0.101, z: -0.104}, orientation: Quaternionf {x: -0.175, y: -0.809, z: 0.371,w:  -0.420}},
        Posef { position: Vector3f {x: -0.599, y: -0.089, z: -0.092}, orientation: Quaternionf {x: -0.109, y: -0.856, z: 0.245,w:  -0.443}},
        Posef { position: Vector3f {x: -0.608, y: -0.084, z: -0.086}, orientation: Quaternionf {x: -0.075, y: -0.871, z: 0.180,w:  -0.450}},
        Posef { position: Vector3f {x: -0.535, y: -0.178, z: -0.152}, orientation: Quaternionf {x: -0.132, y: -0.786, z: 0.408,w:  -0.445}},
        Posef { position: Vector3f {x: -0.568, y: -0.136, z: -0.137}, orientation: Quaternionf {x: -0.132, y: -0.786, z: 0.408,w:  -0.445}},
        Posef { position: Vector3f {x: -0.590, y: -0.106, z: -0.130}, orientation: Quaternionf {x: -0.131, y: -0.762, z: 0.432,w:  -0.464}},
        Posef { position: Vector3f {x: -0.607, y: -0.092, z: -0.122}, orientation: Quaternionf {x: -0.071, y: -0.810, z: 0.332,w:  -0.477}},
        Posef { position: Vector3f {x: -0.617, y: -0.086, z: -0.117}, orientation: Quaternionf {x: -0.029, y: -0.836, z: 0.260,w:  -0.482}},
        Posef { position: Vector3f {x: -0.544, y: -0.183, z: -0.159}, orientation: Quaternionf {x: -0.060, y: -0.749, z: 0.481,w:  -0.452}},
        Posef { position: Vector3f {x: -0.576, y: -0.143, z: -0.152}, orientation: Quaternionf {x: -0.060, y: -0.749, z: 0.481,w:  -0.452}},
        Posef { position: Vector3f {x: -0.594, y: -0.119, z: -0.154}, orientation: Quaternionf {x: -0.061, y: -0.684, z: 0.534,w:  -0.493}},
        Posef { position: Vector3f {x: -0.607, y: -0.108, z: -0.152}, orientation: Quaternionf {x: 0.002, y: -0.745, z: 0.444,w:  -0.498}}, 
        Posef { position: Vector3f {x: -0.616, y: -0.102, z: -0.150}, orientation: Quaternionf {x: 0.045, y: -0.780, z: 0.378,w:  -0.496}},
    ];
    //cursed wrist math
    let wrist_dist = Vec3{ x: 0.01, y: -0.05, z: 0.0 };
    let huh = Quaternionf {x: -0.267,y:  0.849, z: 0.204,w:  0.407}.to_quat();
    let why: Vec3 = huh.mul_vec3(wrist_dist);

    let offset = Vec3 { x: 0.548, y: 1.0, z: 0.137 };
    let palm = hand_pose[HandJoint::PALM];
    gizmos.sphere(palm.position.to_vec3() + offset, palm.orientation.to_quat(), 0.01, Color::WHITE);
    let wrist = hand_pose[HandJoint::WRIST];
    gizmos.sphere(palm.position.to_vec3() + offset + why, palm.orientation.to_quat(), 0.01, Color::GRAY);
    let thumb_meta = hand_pose[HandJoint::THUMB_METACARPAL];
    gizmos.sphere(thumb_meta.position.to_vec3() + offset, thumb_meta.orientation.to_quat(), 0.01, Color::RED);
    let thumb_prox = hand_pose[HandJoint::THUMB_PROXIMAL];
    gizmos.sphere(thumb_prox.position.to_vec3() + offset, thumb_prox.orientation.to_quat(), 0.008, Color::RED);
    let thumb_dist = hand_pose[HandJoint::THUMB_DISTAL];
    gizmos.sphere(thumb_dist.position.to_vec3() + offset, thumb_dist.orientation.to_quat(), 0.006, Color::RED);
    let thumb_tip = hand_pose[HandJoint::THUMB_TIP];
    gizmos.sphere(thumb_tip.position.to_vec3() + offset, thumb_tip.orientation.to_quat(), 0.004, Color::RED);

    let index_meta = hand_pose[HandJoint::INDEX_METACARPAL];
    gizmos.sphere(index_meta.position.to_vec3() + offset, index_meta.orientation.to_quat(), 0.01, Color::ORANGE);
    let index_prox = hand_pose[HandJoint::INDEX_PROXIMAL];
    gizmos.sphere(index_prox.position.to_vec3() + offset, index_prox.orientation.to_quat(), 0.008, Color::ORANGE);
    let index_inter = hand_pose[HandJoint::INDEX_INTERMEDIATE];
    gizmos.sphere(index_inter.position.to_vec3() + offset, index_inter.orientation.to_quat(), 0.006, Color::ORANGE);
    let index_dist = hand_pose[HandJoint::INDEX_DISTAL];
    gizmos.sphere(index_dist.position.to_vec3() + offset, index_dist.orientation.to_quat(), 0.004, Color::ORANGE);
    let index_tip = hand_pose[HandJoint::INDEX_TIP];
    gizmos.sphere(index_tip.position.to_vec3() + offset, index_tip.orientation.to_quat(), 0.002, Color::ORANGE);

    let middle_meta = hand_pose[HandJoint::MIDDLE_METACARPAL];
    gizmos.sphere(middle_meta.position.to_vec3() + offset, middle_meta.orientation.to_quat(), 0.01, Color::YELLOW);
    let middle_prox = hand_pose[HandJoint::MIDDLE_PROXIMAL];
    gizmos.sphere(middle_prox.position.to_vec3() + offset, middle_prox.orientation.to_quat(), 0.008, Color::YELLOW);
    let middle_inter = hand_pose[HandJoint::MIDDLE_INTERMEDIATE];
    gizmos.sphere(middle_inter.position.to_vec3() + offset, middle_inter.orientation.to_quat(), 0.006, Color::YELLOW);
    let middle_dist = hand_pose[HandJoint::MIDDLE_DISTAL];
    gizmos.sphere(middle_dist.position.to_vec3() + offset, middle_dist.orientation.to_quat(), 0.004, Color::YELLOW);
    let middle_tip = hand_pose[HandJoint::MIDDLE_TIP];
    gizmos.sphere(middle_tip.position.to_vec3() + offset, middle_tip.orientation.to_quat(), 0.002, Color::YELLOW);

    let ring_meta = hand_pose[HandJoint::RING_METACARPAL];
    gizmos.sphere(ring_meta.position.to_vec3() + offset, ring_meta.orientation.to_quat(), 0.01, Color::GREEN);
    let ring_prox = hand_pose[HandJoint::RING_PROXIMAL];
    gizmos.sphere(ring_prox.position.to_vec3() + offset, ring_prox.orientation.to_quat(), 0.008, Color::GREEN);
    let ring_inter = hand_pose[HandJoint::RING_INTERMEDIATE];
    gizmos.sphere(ring_inter.position.to_vec3() + offset, ring_inter.orientation.to_quat(), 0.006, Color::GREEN);
    let ring_dist = hand_pose[HandJoint::RING_DISTAL];
    gizmos.sphere(ring_dist.position.to_vec3() + offset, ring_dist.orientation.to_quat(), 0.004, Color::GREEN);
    let ring_tip = hand_pose[HandJoint::RING_TIP];
    gizmos.sphere(ring_tip.position.to_vec3() + offset, ring_tip.orientation.to_quat(), 0.002, Color::GREEN);

    let little_meta = hand_pose[HandJoint::LITTLE_METACARPAL];
    gizmos.sphere(little_meta.position.to_vec3() + offset, little_meta.orientation.to_quat(), 0.01, Color::BLUE);
    let little_prox = hand_pose[HandJoint::LITTLE_PROXIMAL];
    gizmos.sphere(little_prox.position.to_vec3() + offset, little_prox.orientation.to_quat(), 0.008, Color::BLUE);
    let little_inter = hand_pose[HandJoint::LITTLE_INTERMEDIATE];
    gizmos.sphere(little_inter.position.to_vec3() + offset, little_inter.orientation.to_quat(), 0.006, Color::BLUE);
    let little_dist = hand_pose[HandJoint::LITTLE_DISTAL];
    gizmos.sphere(little_dist.position.to_vec3() + offset, little_dist.orientation.to_quat(), 0.004, Color::BLUE);
    let little_tip = hand_pose[HandJoint::LITTLE_TIP];
    gizmos.sphere(little_tip.position.to_vec3() + offset, little_tip.orientation.to_quat(), 0.002, Color::BLUE);
        
    
    



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
