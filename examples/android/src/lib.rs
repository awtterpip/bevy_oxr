use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_oxr::graphics::extensions::XrExtensions;
use bevy_oxr::graphics::XrAppInfo;
use bevy_oxr::passthrough::{PausePassthrough, ResumePassthrough, XrPassthroughState};
use bevy_oxr::xr_init::xr_only;
use bevy_oxr::xr_input::hands::common::HandInputDebugRenderer;
use bevy_oxr::xr_input::hands::HandBone;
use bevy_oxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_oxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
};
use bevy_oxr::DefaultXrPlugins;

#[bevy_main]
fn main() {
    let mut xr_extensions = XrExtensions::default();
    xr_extensions.enable_fb_passthrough();
    xr_extensions.enable_hand_tracking();
    App::new()
        .add_plugins(DefaultXrPlugins {
            reqeusted_extensions: xr_extensions,
            app_info: XrAppInfo {
                name: "Bevy OXR Android Example".into(),
            },
            enable_pipelined_rendering: true,
            ..Default::default()
        })
        // .add_plugins(OpenXrDebugRenderer)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(HandInputDebugRenderer)
        .add_plugins(bevy_oxr::passthrough::EnablePassthroughStartup)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (proto_locomotion, toggle_passthrough).run_if(xr_only()),
        )
        .add_systems(Update, debug_hand_render.run_if(xr_only()))
        .add_systems(Startup, spawn_controllers_example)
        .insert_resource(PrototypeLocomotionConfig::default())
        .run();
}

fn debug_hand_render(query: Query<&GlobalTransform, With<HandBone>>, mut gizmos: Gizmos) {
    for transform in &query {
        gizmos.sphere(transform.translation(), Quat::IDENTITY, 0.01, Color::RED);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::new(Vec3::Y)),
        material: materials.add(StandardMaterial::from(Color::rgb(0.3, 0.5, 0.3))),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::from_size(Vec3::splat(0.1)).mesh()),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.7, 0.6))),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(0.1)))),
        material: materials.add(StandardMaterial::from(Color::rgb(0.8, 0.0, 0.0))),
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

// TODO: make this a vr button
fn toggle_passthrough(
    keys: Res<ButtonInput<KeyCode>>,
    passthrough_state: Res<XrPassthroughState>,
    mut resume: EventWriter<ResumePassthrough>,
    mut pause: EventWriter<PausePassthrough>,
) {
    if keys.just_pressed(KeyCode::Space) {
        match *passthrough_state {
            XrPassthroughState::Unsupported => {}
            XrPassthroughState::Running => {
                pause.send_default();
            }
            XrPassthroughState::Paused => {
                resume.send_default();
            }
        }
    }
}
