

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::prelude::*;
use bevy::transform::components::Transform;

use bevy_openxr::xr_input::{QuatConv, Vec3Conv};
use bevy_openxr::xr_input::hand::{OpenXrHandInput, HandInputDebugRenderer};
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
        //.add_plugins(OpenXrDebugRenderer) //new debug renderer adds gizmos to
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, proto_locomotion)
        .insert_resource(PrototypeLocomotionConfig::default())
        .add_systems(Startup, spawn_controllers_example)
        .add_plugins(OpenXrHandInput)
        .add_plugins(HandInputDebugRenderer)
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

fn draw_skeleton_hand(mut commands: Commands,
    mut gizmos: Gizmos,
    right_controller_query: Query<(
    &GlobalTransform,
    With<OpenXRRightController>,
)>, ) {
    //draw debug for controller grip center to match palm to
    let right_transform = right_controller_query.get_single().unwrap().0;
    let right_translation = right_transform.compute_transform().translation;
    let right_quat = right_transform.compute_transform().rotation;
    //we need to flip this i dont know why
    let flip = Quat::from_rotation_x(PI);
    let flap = Quat::from_rotation_z(-45.0*(PI/180.0));
    let controller_backward = right_quat.mul_quat(flip).mul_quat(flap);

    let hand_pose: [Posef; 26] = [
        Posef { position: Vector3f {x: 0.0, y: 0.0, z: 0.0}, orientation: Quaternionf {x: -0.267, y:  0.849, z: 0.204, w:  0.407}}, //palm
        Posef { position: Vector3f {x: 0.0, y: 0.32,  z: 0.0}, orientation: Quaternionf {x: -0.267, y:  0.849, z: 0.204, w:  0.407}},

        Posef { position: Vector3f {x: 0.019, y: -0.037, z: 0.011}, orientation: Quaternionf {x: -0.744, y: -0.530, z: 0.156, w:  -0.376}},
        Posef { position: Vector3f {x: 0.015, y: -0.014, z: 0.047}, orientation: Quaternionf {x: -0.786, y: -0.550, z: 0.126, w:  -0.254}},
        Posef { position: Vector3f {x: 0.004, y: 0.003, z: 0.068}, orientation: Quaternionf {x: -0.729, y: -0.564, z: 0.027, w:  -0.387}}, 
        Posef { position: Vector3f {x: -0.009, y: 0.011, z: 0.072}, orientation: Quaternionf {x: -0.585, y: -0.548, z: -0.140,w:  -0.582}},

        Posef { position: Vector3f {x: 0.027, y: -0.021, z: 0.001}, orientation: Quaternionf {x: -0.277, y: -0.826, z: 0.317, w:  -0.376}},
        Posef { position: Vector3f {x: -0.002, y:0.026, z:0.034}, orientation: Quaternionf {x: -0.277, y: -0.826, z: 0.317, w:  -0.376}},
        Posef { position: Vector3f {x: -0.023, y:0.049, z:0.055}, orientation: Quaternionf {x: -0.244, y: -0.843, z: 0.256, w:  -0.404}},
        Posef { position: Vector3f {x: -0.037, y:0.059, z:0.067}, orientation: Quaternionf {x: -0.200, y: -0.866, z: 0.165, w:  -0.428}}, 
        Posef { position: Vector3f {x: -0.045, y:0.063, z:0.073}, orientation: Quaternionf {x: -0.172, y: -0.874, z: 0.110, w:  -0.440}},

        Posef { position: Vector3f {x: 0.021, y: -0.017, z: -0.007}, orientation: Quaternionf {x: -0.185, y: -0.817, z: 0.370, w:  -0.401}},
        Posef { position: Vector3f {x: -0.011, y: 0.029, z:0.018}, orientation: Quaternionf {x: -0.185, y: -0.817, z: 0.370, w:  -0.401}},
        Posef { position: Vector3f {x: -0.034, y:0.06, z:0.033}, orientation: Quaternionf {x: -0.175, y: -0.809, z: 0.371, w:  -0.420}},
        Posef { position: Vector3f {x: -0.051, y: 0.072, z: 0.045}, orientation: Quaternionf {x: -0.109, y: -0.856, z: 0.245, w:  -0.443}},
        Posef { position: Vector3f {x: -0.06, y: 0.077, z:0.051}, orientation: Quaternionf {x: -0.075, y: -0.871, z: 0.180, w:  -0.450}},

        Posef { position: Vector3f {x: 0.013, y:-0.017, z:-0.015}, orientation: Quaternionf {x: -0.132, y: -0.786, z: 0.408, w:  -0.445}},
        Posef { position: Vector3f {x: -0.02, y: 0.025, z: 0.0}, orientation: Quaternionf {x: -0.132, y: -0.786, z: 0.408, w:  -0.445}},
        Posef { position: Vector3f {x: -0.042, y:0.055, z:0.007}, orientation: Quaternionf {x: -0.131, y: -0.762, z: 0.432, w:  -0.464}},
        Posef { position: Vector3f {x: -0.06, y:0.069, z: 0.015}, orientation: Quaternionf {x: -0.071, y: -0.810, z: 0.332, w:  -0.477}},
        Posef { position: Vector3f {x: -0.069, y:0.075, z:0.02}, orientation: Quaternionf {x: -0.029, y: -0.836, z: 0.260, w:  -0.482}},

        Posef { position: Vector3f {x: 0.004, y:-0.022, z:-0.022}, orientation: Quaternionf {x: -0.060, y: -0.749, z: 0.481, w:  -0.452}},
        Posef { position: Vector3f {x: -0.028, y:0.018, z:-0.015}, orientation: Quaternionf {x: -0.060, y: -0.749, z: 0.481, w:  -0.452}},
        Posef { position: Vector3f {x: -0.046, y:0.042, z:-0.017}, orientation: Quaternionf {x: -0.061, y: -0.684, z: 0.534, w:  -0.493}},
        Posef { position: Vector3f {x: -0.059, y:0.053, z:-0.015}, orientation: Quaternionf {x: 0.002,  y: -0.745, z: 0.444, w:  -0.498}}, 
        Posef { position: Vector3f {x: -0.068, y:0.059, z:-0.013}, orientation: Quaternionf {x: 0.045,  y: -0.780, z: 0.378, w:  -0.496}},
    ];

    //log_hand(hand_pose);


    //cursed wrist math
    let wrist_dist = Vec3{ x: -0.01, y: -0.040, z: -0.015};

    //cursed offset
    let offset = right_translation;

    //old stuff dont touch for now
    let palm = hand_pose[HandJoint::PALM];
    gizmos.sphere(palm.position.to_vec3() + offset, palm.orientation.to_quat().mul_quat(controller_backward), 0.01, Color::WHITE);

    let rotated_wfp =  palm.position.to_vec3()  + right_quat.mul_quat(flip).mul_vec3(wrist_dist);
    gizmos.sphere(offset + rotated_wfp, palm.orientation.to_quat(), 0.01, Color::GRAY);


    let thumb_meta = hand_pose[HandJoint::THUMB_METACARPAL];
    draw_joint(&mut gizmos, thumb_meta.position.to_vec3(), thumb_meta.orientation.to_quat(), 0.01, Color::RED, controller_backward, offset);

    let thumb_prox = hand_pose[HandJoint::THUMB_PROXIMAL];
    draw_joint(&mut gizmos, thumb_prox.position.to_vec3(), thumb_prox.orientation.to_quat(), 0.008, Color::RED, controller_backward, offset);
    let thumb_dist = hand_pose[HandJoint::THUMB_DISTAL];
    draw_joint(&mut gizmos, thumb_dist.position.to_vec3(), thumb_dist.orientation.to_quat(), 0.006, Color::RED, controller_backward, offset);
    let thumb_tip = hand_pose[HandJoint::THUMB_TIP];
    draw_joint(&mut gizmos, thumb_tip.position.to_vec3(), thumb_tip.orientation.to_quat(), 0.004, Color::RED, controller_backward, offset);

    let index_meta = hand_pose[HandJoint::INDEX_METACARPAL];
    draw_joint(&mut gizmos, index_meta.position.to_vec3(), index_meta.orientation.to_quat(), 0.01, Color::ORANGE, controller_backward, offset);
    let index_prox = hand_pose[HandJoint::INDEX_PROXIMAL];
    draw_joint(&mut gizmos, index_prox.position.to_vec3(), index_prox.orientation.to_quat(), 0.008, Color::ORANGE, controller_backward, offset);
    let index_inter = hand_pose[HandJoint::INDEX_INTERMEDIATE];
    draw_joint(&mut gizmos, index_inter.position.to_vec3(), index_inter.orientation.to_quat(), 0.006, Color::ORANGE, controller_backward, offset);
    let index_dist = hand_pose[HandJoint::INDEX_DISTAL];
    draw_joint(&mut gizmos, index_dist.position.to_vec3(), index_dist.orientation.to_quat(), 0.004, Color::ORANGE, controller_backward, offset);
    let index_tip = hand_pose[HandJoint::INDEX_TIP];
    draw_joint(&mut gizmos, index_tip.position.to_vec3(), index_tip.orientation.to_quat(), 0.002, Color::ORANGE, controller_backward, offset);

    let middle_meta = hand_pose[HandJoint::MIDDLE_METACARPAL];
    draw_joint(&mut gizmos, middle_meta.position.to_vec3(), middle_meta.orientation.to_quat(), 0.01, Color::YELLOW, controller_backward, offset);
    let middle_prox = hand_pose[HandJoint::MIDDLE_PROXIMAL];
    draw_joint(&mut gizmos, middle_prox.position.to_vec3(), middle_prox.orientation.to_quat(), 0.008, Color::YELLOW, controller_backward, offset);
    let middle_inter = hand_pose[HandJoint::MIDDLE_INTERMEDIATE];
    draw_joint(&mut gizmos, middle_inter.position.to_vec3(), middle_inter.orientation.to_quat(), 0.006, Color::YELLOW, controller_backward, offset);
    let middle_dist = hand_pose[HandJoint::MIDDLE_DISTAL];
    draw_joint(&mut gizmos, middle_dist.position.to_vec3(), middle_dist.orientation.to_quat(), 0.004, Color::YELLOW, controller_backward, offset);
    let middle_tip = hand_pose[HandJoint::MIDDLE_TIP];
    draw_joint(&mut gizmos, middle_tip.position.to_vec3(), middle_tip.orientation.to_quat(), 0.002, Color::YELLOW, controller_backward, offset);

    let ring_meta = hand_pose[HandJoint::RING_METACARPAL];
    draw_joint(&mut gizmos, ring_meta.position.to_vec3(), ring_meta.orientation.to_quat(), 0.01, Color::GREEN, controller_backward, offset);
    let ring_prox = hand_pose[HandJoint::RING_PROXIMAL];
    draw_joint(&mut gizmos, ring_prox.position.to_vec3(), ring_prox.orientation.to_quat(), 0.008, Color::GREEN, controller_backward, offset);
    let ring_inter = hand_pose[HandJoint::RING_INTERMEDIATE];
    draw_joint(&mut gizmos, ring_inter.position.to_vec3(), ring_inter.orientation.to_quat(), 0.006, Color::GREEN, controller_backward, offset);
    let ring_dist = hand_pose[HandJoint::RING_DISTAL];
    draw_joint(&mut gizmos, ring_dist.position.to_vec3(), ring_dist.orientation.to_quat(), 0.004, Color::GREEN, controller_backward, offset);
    let ring_tip = hand_pose[HandJoint::RING_TIP];
    draw_joint(&mut gizmos, ring_tip.position.to_vec3(), ring_tip.orientation.to_quat(), 0.002, Color::GREEN, controller_backward, offset);

    let little_meta = hand_pose[HandJoint::LITTLE_METACARPAL];
    draw_joint(&mut gizmos, little_meta.position.to_vec3(), little_meta.orientation.to_quat(), 0.01, Color::BLUE, controller_backward, offset);
    let little_prox = hand_pose[HandJoint::LITTLE_PROXIMAL];
    draw_joint(&mut gizmos, little_prox.position.to_vec3(), little_prox.orientation.to_quat(), 0.008, Color::BLUE, controller_backward, offset);
    let little_inter = hand_pose[HandJoint::LITTLE_INTERMEDIATE];
    draw_joint(&mut gizmos, little_inter.position.to_vec3(), little_inter.orientation.to_quat(), 0.006, Color::BLUE, controller_backward, offset);
    let little_dist = hand_pose[HandJoint::LITTLE_DISTAL];
    draw_joint(&mut gizmos, little_dist.position.to_vec3(), little_dist.orientation.to_quat(), 0.004, Color::BLUE, controller_backward, offset);
    let little_tip = hand_pose[HandJoint::LITTLE_TIP];
    draw_joint(&mut gizmos, little_tip.position.to_vec3(), little_tip.orientation.to_quat(), 0.002, Color::BLUE, controller_backward, offset);




}

fn draw_joint(gizmos: &mut Gizmos, joint_pos: Vec3, joint_rot: Quat, radius: f32, color: Color, controller_backwards: Quat, offset: Vec3) {
    gizmos.sphere(controller_backwards.mul_vec3(joint_pos) + offset, joint_rot, radius, color);
}

fn log_hand(hand_pose: [Posef; 26]) {
    let palm_vec = hand_pose[HandJoint::PALM].position.to_vec3();
    info!("palm: {}", hand_pose[HandJoint::PALM].position.to_vec3() - palm_vec);
    info!("wrist: {}", hand_pose[HandJoint::WRIST].position.to_vec3() - palm_vec);

    info!("tm: {}", hand_pose[HandJoint::THUMB_METACARPAL].position.to_vec3() - palm_vec);
    info!("tp: {}", hand_pose[HandJoint::THUMB_PROXIMAL].position.to_vec3() - palm_vec);
    info!("td: {}", hand_pose[HandJoint::THUMB_DISTAL].position.to_vec3() - palm_vec);
    info!("tt: {}", hand_pose[HandJoint::THUMB_TIP].position.to_vec3() - palm_vec);
    
    info!("im: {}", hand_pose[HandJoint::INDEX_METACARPAL].position.to_vec3() - palm_vec);
    info!("ip: {}", hand_pose[HandJoint::INDEX_PROXIMAL].position.to_vec3() - palm_vec);
    info!("ii: {}", hand_pose[HandJoint::INDEX_INTERMEDIATE].position.to_vec3() - palm_vec);
    info!("id: {}", hand_pose[HandJoint::INDEX_DISTAL].position.to_vec3() - palm_vec);
    info!("it: {}", hand_pose[HandJoint::INDEX_TIP].position.to_vec3() - palm_vec);

    info!("mm: {}", hand_pose[HandJoint::MIDDLE_METACARPAL].position.to_vec3() - palm_vec);
    info!("mp: {}", hand_pose[HandJoint::MIDDLE_PROXIMAL].position.to_vec3() - palm_vec);
    info!("mi: {}", hand_pose[HandJoint::MIDDLE_INTERMEDIATE].position.to_vec3() - palm_vec);
    info!("md: {}", hand_pose[HandJoint::MIDDLE_DISTAL].position.to_vec3() - palm_vec);
    info!("mt: {}", hand_pose[HandJoint::MIDDLE_TIP].position.to_vec3() - palm_vec);

    info!("rm: {}", hand_pose[HandJoint::RING_METACARPAL].position.to_vec3() - palm_vec);
    info!("rp: {}", hand_pose[HandJoint::RING_PROXIMAL].position.to_vec3() - palm_vec);
    info!("ri: {}", hand_pose[HandJoint::RING_INTERMEDIATE].position.to_vec3() - palm_vec);
    info!("rd: {}", hand_pose[HandJoint::RING_DISTAL].position.to_vec3() - palm_vec);
    info!("rt: {}", hand_pose[HandJoint::RING_TIP].position.to_vec3() - palm_vec);

    info!("lm: {}", hand_pose[HandJoint::LITTLE_METACARPAL].position.to_vec3() - palm_vec);
    info!("lp: {}", hand_pose[HandJoint::LITTLE_PROXIMAL].position.to_vec3() - palm_vec);
    info!("li: {}", hand_pose[HandJoint::LITTLE_INTERMEDIATE].position.to_vec3() - palm_vec);
    info!("ld: {}", hand_pose[HandJoint::LITTLE_DISTAL].position.to_vec3() - palm_vec);
    info!("lt: {}", hand_pose[HandJoint::LITTLE_TIP].position.to_vec3() - palm_vec);
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