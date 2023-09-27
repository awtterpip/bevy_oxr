use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession};
use bevy_openxr::xr_input::debug_gizmos::OpenXrDebugRenderer;
use bevy_openxr::xr_input::interactions::{
    draw_interaction_gizmos, direct_interaction, XRDirectInteractor, XRInteractable,
    XRInteractableState, XRInteractorState, XRRayInteractor, ray_interaction,
};
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_openxr::xr_input::trackers::{
    OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker, AimPose,
};
use bevy_openxr::xr_input::Hand;
use bevy_openxr::DefaultXrPlugins;

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
        .insert_resource(PrototypeLocomotionConfig::default())
        .add_systems(Update, draw_interaction_gizmos)
        .add_systems(Update, direct_interaction)
        .add_systems(Update, ray_interaction)
        .add_systems(Update, prototype_interaction_input)
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
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
    //simple interactable
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        XRInteractable,
        XRInteractableState::default(),
    ));
}

fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        XRDirectInteractor,
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

fn prototype_interaction_input(
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut right_interactor_query: Query<
        (&mut XRInteractorState),
        (
            With<XRDirectInteractor>,
            With<OpenXRRightController>,
            Without<OpenXRLeftController>,
        ),
    >,
    mut left_interactor_query: Query<
        (&mut XRInteractorState),
        (
            With<XRDirectInteractor>,
            With<OpenXRLeftController>,
            Without<OpenXRRightController>,
        ),
    >,
) {
    //lock frame
    let frame_state = *frame_state.lock().unwrap();
    //get controller
    let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);
    //get controller triggers
    let left_trigger = controller.trigger(Hand::Left);
    let right_trigger = controller.trigger(Hand::Right);
    //get the interactors and do state stuff
    let mut left_state = left_interactor_query.single_mut();
    if left_trigger > 0.8 {
        *left_state = XRInteractorState::Selecting;
    } else {
        *left_state = XRInteractorState::Idle;
    }
    let mut right_state = right_interactor_query.single_mut();
    if right_trigger > 0.8 {
        *right_state = XRInteractorState::Selecting;
    } else {
        *right_state = XRInteractorState::Idle;
    }
}
