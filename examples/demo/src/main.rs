use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::info,
    prelude::{
        default, shape, App, Assets, Color, Commands, Component, Entity, Event, EventReader,
        EventWriter, GlobalTransform, IntoSystemConfigs, IntoSystemSetConfigs, Mesh, PbrBundle,
        PostUpdate, Query, Res, ResMut, Resource, SpatialBundle, StandardMaterial, Startup,
        Transform, Update, With, Without,
    },
    time::{Time, Timer},
    transform::TransformSystem,
};
use bevy_openxr::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
    xr_input::{
        debug_gizmos::OpenXrDebugRenderer,
        interactions::{
            draw_interaction_gizmos, draw_socket_gizmos, interactions, socket_interactions,
            update_interactable_states, InteractionEvent, Touched, XRDirectInteractor,
            XRInteractable, XRInteractableState, XRInteractorState, XRSelection,
        },
        oculus_touch::OculusController,
        prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig},
        trackers::{OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker},
        Hand,
    },
    DefaultXrPlugins,
};

mod setup;
use crate::setup::setup_scene;
use bevy_rapier3d::prelude::*;

fn main() {
    color_eyre::install().unwrap();

    info!("Running bevy_openxr demo");
    let mut app = App::new();

    app
        //lets get the usual diagnostic stuff added
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        //lets get the xr defaults added
        .add_plugins(DefaultXrPlugins)
        //lets add the debug renderer for the controllers
        .add_plugins(OpenXrDebugRenderer)
        //rapier goes here
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false))
        // .add_plugins(RapierDebugRenderPlugin::default())
        //lets setup the starting scene
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, spawn_controllers_example) //you need to spawn controllers or it crashes TODO:: Fix this
        //add locomotion
        .add_systems(Update, proto_locomotion)
        .insert_resource(PrototypeLocomotionConfig::default())
        //lets add the interaction systems
        .add_event::<InteractionEvent>()
        .add_systems(Update, prototype_interaction_input)
        .add_systems(Update, interactions.before(update_interactable_states))
        .add_systems(Update, update_interactable_states)
        .add_systems(
            Update,
            socket_interactions.before(update_interactable_states),
        )
        //add the grabbable system
        .add_systems(Update, update_grabbables.after(update_interactable_states))
        //draw the interaction gizmos
        .add_systems(
            Update,
            draw_interaction_gizmos.after(update_interactable_states),
        )
        .add_systems(Update, draw_socket_gizmos.after(update_interactable_states))
        //add our cube spawning system
        .add_event::<SpawnCubeRequest>()
        .insert_resource(SpawnCubeTimer(Timer::from_seconds(
            0.25,
            bevy::time::TimerMode::Once,
        )))
        .add_systems(Update, request_cube_spawn)
        .add_systems(Update, cube_spawner.after(request_cube_spawn));

    //configure rapier sets
    app.configure_sets(
        PostUpdate,
        (
            PhysicsSet::SyncBackend,
            PhysicsSet::StepSimulation,
            PhysicsSet::Writeback,
        )
            .chain()
            .before(TransformSystem::TransformPropagate),
    );
    //add rapier systems
    app.add_systems(
        PostUpdate,
        (
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
                .in_set(PhysicsSet::SyncBackend),
            (
                RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation),
                // despawn_one_box,
            )
                .in_set(PhysicsSet::StepSimulation),
            RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
                .in_set(PhysicsSet::Writeback),
        ),
    );

    app.run();
}

fn spawn_controllers_example(mut commands: Commands) {
    //left hand
    commands.spawn((
        OpenXRLeftController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        XRDirectInteractor,
        XRInteractorState::default(),
        XRSelection::default(),
    ));
    //right hand
    commands.spawn((
        OpenXRRightController,
        OpenXRController,
        OpenXRTracker,
        SpatialBundle::default(),
        XRDirectInteractor,
        XRInteractorState::default(),
        XRSelection::default(),
    ));
}

#[derive(Event, Default)]
pub struct SpawnCubeRequest;

#[derive(Resource)]
pub struct SpawnCubeTimer(Timer);

fn request_cube_spawn(
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
    mut writer: EventWriter<SpawnCubeRequest>,
    time: Res<Time>,
    mut timer: ResMut<SpawnCubeTimer>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        //lock frame
        let frame_state = *frame_state.lock().unwrap();
        //get controller
        let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);
        //get controller triggers
        let left_main_button = controller.a_button();
        if left_main_button {
            writer.send(SpawnCubeRequest::default());
            timer.0.reset();
        }
        let right_main_button = controller.x_button();
        if right_main_button {
            writer.send(SpawnCubeRequest::default());
            timer.0.reset();
        }
    }
}

fn cube_spawner(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events: EventReader<SpawnCubeRequest>,
) {
    for request in events.read() {
        // cube
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(0.0, 1.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            Collider::cuboid(0.05, 0.05, 0.05),
            ColliderDebugColor(Color::hsl(220.0, 1.0, 0.3)),
            XRInteractable,
            XRInteractableState::default(),
            Grabbable,
            Touched(false),
        ));
    }
}

//TODO: find a real place for this
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

#[derive(Component)]
pub struct Grabbable;

pub fn update_grabbables(
    mut events: EventReader<InteractionEvent>,
    mut grabbable_query: Query<(
        Entity,
        &mut Transform,
        With<Grabbable>,
        Without<XRDirectInteractor>,
        Option<&mut RigidBody>,
    )>,
    mut interactor_query: Query<(
        &GlobalTransform,
        &XRInteractorState,
        &mut XRSelection,
        Without<Grabbable>,
    )>,
) {
    //so basically the idea is to try all the events?
    for event in events.read() {
        // info!("some event");
        match grabbable_query.get_mut(event.interactable) {
            Ok(mut grabbable_transform) => {
                // info!("we got a grabbable");
                //now we need the location of our interactor
                match interactor_query.get_mut(event.interactor) {
                    Ok(mut interactor_transform) => {
                        match *interactor_transform.2 {
                            XRSelection::Empty => {
                                match interactor_transform.1 {
                                    XRInteractorState::Idle => match grabbable_transform.4 {
                                        Some(mut thing) => {
                                            *thing = RigidBody::Dynamic;
                                            *interactor_transform.2 = XRSelection::Empty;
                                        }
                                        None => (),
                                    },
                                    XRInteractorState::Selecting => {
                                        // info!("its a direct interactor?");
                                        match grabbable_transform.4 {
                                            Some(mut thing) => {
                                                *thing = RigidBody::KinematicPositionBased;
                                                *interactor_transform.2 =
                                                    XRSelection::Full(grabbable_transform.0);
                                            }
                                            None => (),
                                        }
                                        *grabbable_transform.1 =
                                            interactor_transform.0.compute_transform();
                                    }
                                }
                            }
                            XRSelection::Full(ent) => {
                                info!("nah bro we holding something");
                                match grabbable_transform.0 == ent {
                                    true => {
                                        *grabbable_transform.1 =
                                            interactor_transform.0.compute_transform();
                                    }
                                    false => {}
                                }
                                match interactor_transform.1 {
                                    XRInteractorState::Idle => {
                                        *interactor_transform.2 = XRSelection::Empty
                                    }
                                    XRInteractorState::Selecting => {}
                                }
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
