use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_oxr::graphics::XrAppInfo;
use bevy_oxr::input::XrInput;
use bevy_oxr::resources::{XrFrameState, XrSession};

use bevy_oxr::xr_init::{xr_only, EndXrSession, StartXrSession};
use bevy_oxr::xr_input::actions::XrActionSets;
use bevy_oxr::xr_input::hands::common::HandInputDebugRenderer;
use bevy_oxr::xr_input::interactions::{
    draw_interaction_gizmos, draw_socket_gizmos, interactions, socket_interactions,
    update_interactable_states, InteractionEvent, Touched, XRDirectInteractor, XRInteractable,
    XRInteractableState, XRInteractorState, XRRayInteractor, XRSocketInteractor,
};
use bevy_oxr::xr_input::oculus_touch::OculusController;
use bevy_oxr::xr_input::prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig};
use bevy_oxr::xr_input::trackers::{
    AimPose, OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
};
use bevy_oxr::xr_input::Hand;
use bevy_oxr::DefaultXrPlugins;

fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins {
            app_info: XrAppInfo {
                name: "Bevy OXR Example".into(),
            },
            ..default()
        })
        //.add_plugins(OpenXrDebugRenderer) //new debug renderer adds gizmos to
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, proto_locomotion.run_if(xr_only()))
        .insert_resource(PrototypeLocomotionConfig::default())
        .add_systems(Startup, spawn_controllers_example)
        .add_plugins(HandInputDebugRenderer)
        .add_systems(
            Update,
            draw_interaction_gizmos
                .after(update_interactable_states)
                .run_if(xr_only()),
        )
        .add_systems(
            Update,
            draw_socket_gizmos
                .after(update_interactable_states)
                .run_if(xr_only()),
        )
        .add_systems(
            Update,
            interactions
                .before(update_interactable_states)
                .run_if(xr_only()),
        )
        .add_systems(
            Update,
            socket_interactions.before(update_interactable_states),
        )
        .add_systems(Update, prototype_interaction_input.run_if(xr_only()))
        .add_systems(Update, update_interactable_states)
        .add_systems(Update, update_grabbables.after(update_interactable_states))
        .add_systems(Update, start_stop_session)
        .add_event::<InteractionEvent>()
        .run();
}

fn start_stop_session(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut start: EventWriter<StartXrSession>,
    mut stop: EventWriter<EndXrSession>,
) {
    if keyboard.just_pressed(KeyCode::KeyS) {
        start.send_default();
    }
    if keyboard.just_pressed(KeyCode::KeyE) {
        stop.send_default();
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
        mesh: meshes.add(Plane3d::new(Vec3::Y).mesh()),
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
    //simple interactable
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
            ..default()
        },
        XRInteractable,
        XRInteractableState::default(),
        Grabbable,
        Touched(false),
    ));
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

fn prototype_interaction_input(
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
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
            With<XRRayInteractor>,
            With<OpenXRLeftController>,
            Without<OpenXRRightController>,
        ),
    >,
    action_sets: Res<XrActionSets>,
) {
    //get controller
    let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
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

#[derive(Component)]
pub struct Grabbable;

pub fn update_grabbables(
    mut events: EventReader<InteractionEvent>,
    mut grabbable_query: Query<&mut Transform, (With<Grabbable>, Without<XRDirectInteractor>)>,
    interactor_query: Query<(&GlobalTransform, &XRInteractorState), Without<Grabbable>>,
) {
    //so basically the idea is to try all the events?
    for event in events.read() {
        // info!("some event");
        match grabbable_query.get_mut(event.interactable) {
            Ok(mut grabbable_transform) => {
                // info!("we got a grabbable");
                //now we need the location of our interactor
                match interactor_query.get(event.interactor) {
                    Ok(interactor_transform) => {
                        match interactor_transform.1 {
                            XRInteractorState::Idle => (),
                            XRInteractorState::Selecting => {
                                // info!("its a direct interactor?");
                                *grabbable_transform = interactor_transform.0.compute_transform();
                            }
                        }
                    }
                    Err(_) => {
                        // info!("not a direct interactor")
                    }
                }
            }
            Err(_) => {
                // info!("not a grabbable?")
            }
        }
    }
}
