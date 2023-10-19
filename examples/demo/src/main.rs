use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::info,
    prelude::{
        App, Commands, IntoSystemConfigs, IntoSystemSetConfigs, PostUpdate, Query, Res,
        SpatialBundle, Startup, Update, With, Without, Component, EventReader, Transform, GlobalTransform,
    },
    transform::TransformSystem,
};
use bevy_openxr::{
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
    xr_input::{
        debug_gizmos::OpenXrDebugRenderer,
        interactions::{
            interactions, update_interactable_states, XRDirectInteractor, XRInteractorState,
            XRRayInteractor, draw_interaction_gizmos, draw_socket_gizmos, socket_interactions, InteractionEvent,
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
        .add_plugins(RapierDebugRenderPlugin::default())
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
        ;

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
    mut grabbable_query: Query<(&mut Transform, With<Grabbable>, Without<XRDirectInteractor>, Option<&mut RigidBody>)>,
    interactor_query: Query<(&GlobalTransform, &XRInteractorState, Without<Grabbable>)>,
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
                            XRInteractorState::Idle => {
                                match grabbable_transform.3 {
                                    Some(mut thing) => {
                                        *thing = RigidBody::Dynamic;
                                    },
                                    None => (),
                                }
                            },
                            XRInteractorState::Selecting => {
                                // info!("its a direct interactor?");
                                match grabbable_transform.3 {
                                    Some(mut thing) => {
                                        *thing = RigidBody::KinematicPositionBased;
                                    },
                                    None => (),
                                }
                                *grabbable_transform.0 = interactor_transform.0.compute_transform();
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
