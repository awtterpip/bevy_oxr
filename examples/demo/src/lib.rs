use std::{f32::consts::PI, ops::Mul, time::Duration};

use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    ecs::schedule::ScheduleLabel,
    input::{keyboard::KeyCode, Input},
    log::info,
    prelude::{
        bevy_main, default, shape, App, Assets, Color, Commands, Component, Entity, Event,
        EventReader, EventWriter, FixedUpdate, Gizmos, GlobalTransform, IntoSystemConfigs,
        IntoSystemSetConfigs, Mesh, PbrBundle, PostUpdate, Quat, Query, Res, ResMut, Resource,
        Schedule, SpatialBundle, StandardMaterial, Startup, Transform, Update, Vec3, Vec3Swizzles,
        With, Without, World,
    },
    time::{Fixed, Time, Timer, TimerMode},
    transform::TransformSystem,
};
use bevy_oxr::{
    graphics::{extensions::XrExtensions, XrAppInfo, XrPreferdBlendMode},
    input::XrInput,
    resources::{XrFrameState, XrInstance, XrSession},
    xr_init::{xr_only, XrEnableRequest, XrEnableStatus},
    xr_input::{
        actions::XrActionSets,
        debug_gizmos::OpenXrDebugRenderer,
        hands::common::{HandInputDebugRenderer, HandResource, HandsResource, OpenXrHandInput},
        hands::HandBone,
        interactions::{
            draw_interaction_gizmos, draw_socket_gizmos, interactions, socket_interactions,
            update_interactable_states, InteractionEvent, Touched, XRDirectInteractor,
            XRInteractable, XRInteractableState, XRInteractorState, XRSelection,
        },
        oculus_touch::OculusController,
        prototype_locomotion::{proto_locomotion, PrototypeLocomotionConfig},
        trackers::{
            OpenXRController, OpenXRLeftController, OpenXRRightController, OpenXRTracker,
            OpenXRTrackingRoot,
        },
        Hand,
    },
    DefaultXrPlugins,
};

fn input_stuff(
    keys: Res<Input<KeyCode>>,
    status: Res<XrEnableStatus>,
    mut request: EventWriter<XrEnableRequest>,
) {
    if keys.just_pressed(KeyCode::Space) {
        match status.into_inner() {
            XrEnableStatus::Enabled => request.send(XrEnableRequest::TryDisable),
            XrEnableStatus::Disabled => request.send(XrEnableRequest::TryEnable),
            XrEnableStatus::Waiting => (),
        }
    }
}

mod setup;
use crate::setup::setup_scene;
use bevy_rapier3d::prelude::*;

#[bevy_main]
pub fn main() {
    color_eyre::install().unwrap();

    info!("Running bevy_openxr demo");
    let mut app = App::new();
    let mut xr_extensions = XrExtensions::default();

    app.add_systems(Update, input_stuff)
        //lets get the usual diagnostic stuff added
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        //lets get the xr defaults added
        .add_plugins(DefaultXrPlugins {
            reqeusted_extensions: xr_extensions,
            prefered_blend_mode: XrPreferdBlendMode::Opaque,
            app_info: XrAppInfo {
                name: "Bevy OXR Demo".into(),
            },
        })
        //lets add the debug renderer for the controllers
        .add_plugins(OpenXrDebugRenderer)
        //rapier goes here
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default().with_default_system_setup(false))
        // .add_plugins(RapierDebugRenderPlugin::default())
        //lets setup the starting scene
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, spawn_controllers_example) //you need to spawn controllers or it crashes TODO:: Fix this
        //add locomotion
        .add_systems(Update, proto_locomotion.run_if(xr_only()))
        .insert_resource(PrototypeLocomotionConfig::default())
        //lets add the interaction systems
        .add_event::<InteractionEvent>()
        .add_systems(Update, prototype_interaction_input.run_if(xr_only()))
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
            draw_interaction_gizmos
                .run_if(xr_only())
                .after(update_interactable_states),
        )
        .add_systems(Update, draw_socket_gizmos.after(update_interactable_states))
        //add our cube spawning system
        .add_event::<SpawnCubeRequest>()
        .insert_resource(SpawnCubeTimer(Timer::from_seconds(
            0.25,
            bevy::time::TimerMode::Once,
        )))
        .add_systems(Update, request_cube_spawn.run_if(xr_only()))
        .add_systems(Update, cube_spawner.after(request_cube_spawn))
        //test capsule
        .add_systems(Startup, spawn_capsule)
        //physics hands
        .add_plugins(OpenXrHandInput)
        .add_plugins(HandInputDebugRenderer)
        .add_systems(Startup, spawn_physics_hands)
        .add_systems(
            FixedUpdate,
            update_physics_hands.before(PhysicsSet::SyncBackend),
        )
        .add_event::<GhostHandEvent>()
        .add_systems(Update, handle_ghost_hand_events.after(update_grabbables))
        .insert_resource(GhostTimers {
            left: Timer::from_seconds(0.25, TimerMode::Once),
            right: Timer::from_seconds(0.25, TimerMode::Once),
        })
        .add_systems(Update, watch_ghost_timers.before(handle_ghost_hand_events));

    //configure rapier sets
    let mut physics_schedule = Schedule::new(PhysicsSchedule);

    physics_schedule.configure_sets(
        (
            PhysicsSet::SyncBackend,
            PhysicsSet::StepSimulation,
            PhysicsSet::Writeback,
        )
            .chain()
            .before(TransformSystem::TransformPropagate),
    );

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
    physics_schedule.add_systems((
        RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::SyncBackend)
            .in_set(PhysicsSet::SyncBackend),
        RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::StepSimulation)
            .in_set(PhysicsSet::StepSimulation),
        RapierPhysicsPlugin::<NoUserData>::get_systems(PhysicsSet::Writeback)
            .in_set(PhysicsSet::Writeback),
    ));
    app.add_schedule(physics_schedule) // configure our fixed timestep schedule to run at the rate we want
        .insert_resource(Time::<Fixed>::from_duration(Duration::from_secs_f32(
            FIXED_TIMESTEP,
        )))
        .add_systems(FixedUpdate, run_physics_schedule)
        .add_systems(Startup, configure_physics);
    app.run();
}

//fixed timesteps?
const FIXED_TIMESTEP: f32 = 1. / 90.;

// A label for our new Schedule!
#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct PhysicsSchedule;

fn run_physics_schedule(world: &mut World) {
    world.run_schedule(PhysicsSchedule);
}

fn configure_physics(mut rapier_config: ResMut<RapierConfiguration>) {
    rapier_config.timestep_mode = TimestepMode::Fixed {
        dt: FIXED_TIMESTEP,
        substeps: 1,
    }
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
        Hand::Left,
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
        Hand::Right,
    ));
}

fn spawn_capsule(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Capsule {
                radius: 0.033,
                depth: 0.115,
                ..default()
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 2.0, 0.0),
            ..default()
        },
        // Collider::capsule_y(0.0575, 0.034),
        Collider::capsule(
            Vec3 {
                x: 0.0,
                y: -0.0575,
                z: 0.0,
            },
            Vec3 {
                x: 0.0,
                y: 0.0575,
                z: 0.0,
            },
            0.034,
        ),
        RigidBody::Dynamic,
    ));
}

#[derive(Component, PartialEq, Debug, Clone, Copy)]
pub enum PhysicsHandBone {
    Palm,
    Wrist,
    ThumbMetacarpal,
    ThumbProximal,
    ThumbDistal,
    ThumbTip,
    IndexMetacarpal,
    IndexProximal,
    IndexIntermediate,
    IndexDistal,
    IndexTip,
    MiddleMetacarpal,
    MiddleProximal,
    MiddleIntermediate,
    MiddleDistal,
    MiddleTip,
    RingMetacarpal,
    RingProximal,
    RingIntermediate,
    RingDistal,
    RingTip,
    LittleMetacarpal,
    LittleProximal,
    LittleIntermediate,
    LittleDistal,
    LittleTip,
}
#[derive(Component, PartialEq)]
pub enum BoneInitState {
    True,
    False,
}

fn spawn_physics_hands(mut commands: Commands) {
    //here we go
    let hands = [Hand::Left, Hand::Right];
    let bones = [
        PhysicsHandBone::Palm,
        PhysicsHandBone::Wrist,
        PhysicsHandBone::ThumbMetacarpal,
        PhysicsHandBone::ThumbProximal,
        PhysicsHandBone::ThumbDistal,
        PhysicsHandBone::ThumbTip,
        PhysicsHandBone::IndexMetacarpal,
        PhysicsHandBone::IndexProximal,
        PhysicsHandBone::IndexIntermediate,
        PhysicsHandBone::IndexDistal,
        PhysicsHandBone::IndexTip,
        PhysicsHandBone::MiddleMetacarpal,
        PhysicsHandBone::MiddleProximal,
        PhysicsHandBone::MiddleIntermediate,
        PhysicsHandBone::MiddleDistal,
        PhysicsHandBone::MiddleTip,
        PhysicsHandBone::RingMetacarpal,
        PhysicsHandBone::RingProximal,
        PhysicsHandBone::RingIntermediate,
        PhysicsHandBone::RingDistal,
        PhysicsHandBone::RingTip,
        PhysicsHandBone::LittleMetacarpal,
        PhysicsHandBone::LittleProximal,
        PhysicsHandBone::LittleIntermediate,
        PhysicsHandBone::LittleDistal,
        PhysicsHandBone::LittleTip,
    ];
    let radius = 0.010;
    let left_hand_membership_group = Group::GROUP_1;
    let right_hand_membership_group = Group::GROUP_2;
    let floor_membership = Group::GROUP_3;

    for hand in hands.iter() {
        let hand_membership = match hand {
            Hand::Left => left_hand_membership_group,
            Hand::Right => right_hand_membership_group,
        };
        let mut hand_filter: Group = Group::ALL;
        hand_filter.remove(hand_membership);
        hand_filter.remove(floor_membership);
        for bone in bones.iter() {
            //spawn the thing
            commands.spawn((
                SpatialBundle::default(),
                Collider::capsule(
                    Vec3 {
                        x: 0.0,
                        y: -0.0575,
                        z: 0.0,
                    },
                    Vec3 {
                        x: 0.0,
                        y: 0.0575,
                        z: 0.0,
                    },
                    radius,
                ),
                RigidBody::Dynamic,
                Velocity::default(),
                CollisionGroups::new(hand_membership, Group::from_bits(0b0001).unwrap()),
                // SolverGroups::new(self_group, interaction_group),
                bone.clone(),
                BoneInitState::False,
                hand.clone(),
            ));
        }
    }
}

pub enum MatchingType {
    PositionMatching,
    VelocityMatching,
}

fn update_physics_hands(
    hands_res: Option<Res<HandsResource>>,
    mut bone_query: Query<(
        &mut Transform,
        &mut Collider,
        &PhysicsHandBone,
        &mut BoneInitState,
        &Hand,
        &mut Velocity,
    )>,
    hand_query: Query<(&Transform, &HandBone, &Hand, Without<PhysicsHandBone>)>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    let matching = MatchingType::VelocityMatching;
    //sanity check do we even have hands?
    match hands_res {
        Some(res) => {
            //config stuff
            let radius = 0.010;
            for mut bone in bone_query.iter_mut() {
                let hand_res = match bone.4 {
                    Hand::Left => res.left,
                    Hand::Right => res.right,
                };

                //lets just do the Right ThumbMetacarpal for now
                let result = get_start_and_end_entities(hand_res, bone.2);
                if let Some((start_entity, end_entity)) = result {
                    //now we need their transforms
                    let start_components = hand_query.get(start_entity);
                    let end_components = hand_query.get(end_entity);
                    let direction = end_components.unwrap().0.translation
                        - start_components.unwrap().0.translation;
                    if direction.length() < 0.001 {
                        //i hate this but we need to skip init if the length is zero
                        return;
                    }

                    match *bone.3 {
                        BoneInitState::True => {
                            match matching {
                                MatchingType::PositionMatching => {
                                    //if we are init then we just move em?
                                    *bone.0 = start_components
                                        .unwrap()
                                        .0
                                        .clone()
                                        .looking_at(end_components.unwrap().0.translation, Vec3::Y);
                                }
                                MatchingType::VelocityMatching => {
                                    //calculate position difference
                                    let diff = (start_components.unwrap().0.translation
                                        - bone.0.translation)
                                        / time.delta_seconds();
                                    bone.5.linvel = diff;
                                    //calculate angular velocity?
                                    // gizmos.ray(bone.0.translation, bone.0.forward(), Color::WHITE);
                                    let desired_forward = start_components
                                        .unwrap()
                                        .0
                                        .clone()
                                        .looking_at(end_components.unwrap().0.translation, Vec3::Y)
                                        .rotation;
                                    // gizmos.ray(
                                    //     bone.0.translation,
                                    //     desired_forward.mul_vec3(-Vec3::Z),
                                    //     Color::GREEN,
                                    // );
                                    let cross =
                                        bone.0.forward().cross(desired_forward.mul_vec3(-Vec3::Z));

                                    // gizmos.ray(
                                    //     bone.0.translation,
                                    //     cross,
                                    //     Color::RED,
                                    // );
                                    bone.5.angvel = cross / time.delta_seconds();
                                }
                            }
                        }
                        BoneInitState::False => {
                            //build a new collider?
                            *bone.1 = Collider::capsule(
                                Vec3::splat(0.0),
                                Vec3 {
                                    x: 0.0,
                                    y: 0.0,
                                    z: -direction.length(),
                                },
                                radius,
                            );
                            *bone.3 = BoneInitState::True;
                        }
                    }
                }
            }
        }
        None => info!("hand states resource not initialized yet"),
    }
}

fn get_start_and_end_entities(
    hand_res: HandResource,
    bone: &PhysicsHandBone,
) -> Option<(Entity, Entity)> {
    match bone {
        PhysicsHandBone::Palm => return None,
        PhysicsHandBone::Wrist => return None,
        PhysicsHandBone::ThumbMetacarpal => {
            return Some((hand_res.thumb.metacarpal, hand_res.thumb.proximal))
        }
        PhysicsHandBone::ThumbProximal => {
            return Some((hand_res.thumb.proximal, hand_res.thumb.distal))
        }
        PhysicsHandBone::ThumbDistal => return Some((hand_res.thumb.distal, hand_res.thumb.tip)),
        PhysicsHandBone::ThumbTip => return None,
        PhysicsHandBone::IndexMetacarpal => {
            return Some((hand_res.index.metacarpal, hand_res.index.proximal))
        }
        PhysicsHandBone::IndexProximal => {
            return Some((hand_res.index.proximal, hand_res.index.intermediate))
        }
        PhysicsHandBone::IndexIntermediate => {
            return Some((hand_res.index.intermediate, hand_res.index.distal))
        }
        PhysicsHandBone::IndexDistal => return Some((hand_res.index.distal, hand_res.index.tip)),
        PhysicsHandBone::IndexTip => return None,
        PhysicsHandBone::MiddleMetacarpal => {
            return Some((hand_res.middle.metacarpal, hand_res.middle.proximal))
        }
        PhysicsHandBone::MiddleProximal => {
            return Some((hand_res.middle.proximal, hand_res.middle.intermediate))
        }
        PhysicsHandBone::MiddleIntermediate => {
            return Some((hand_res.middle.intermediate, hand_res.middle.distal))
        }
        PhysicsHandBone::MiddleDistal => {
            return Some((hand_res.middle.distal, hand_res.middle.tip))
        }
        PhysicsHandBone::MiddleTip => return None,
        PhysicsHandBone::RingMetacarpal => {
            return Some((hand_res.ring.metacarpal, hand_res.ring.proximal))
        }
        PhysicsHandBone::RingProximal => {
            return Some((hand_res.ring.proximal, hand_res.ring.intermediate))
        }
        PhysicsHandBone::RingIntermediate => {
            return Some((hand_res.ring.intermediate, hand_res.ring.distal))
        }
        PhysicsHandBone::RingDistal => return Some((hand_res.ring.distal, hand_res.ring.tip)),
        PhysicsHandBone::RingTip => return None,
        PhysicsHandBone::LittleMetacarpal => {
            return Some((hand_res.little.metacarpal, hand_res.little.proximal))
        }
        PhysicsHandBone::LittleProximal => {
            return Some((hand_res.little.proximal, hand_res.little.intermediate))
        }
        PhysicsHandBone::LittleIntermediate => {
            return Some((hand_res.little.intermediate, hand_res.little.distal))
        }
        PhysicsHandBone::LittleDistal => {
            return Some((hand_res.little.distal, hand_res.little.tip))
        }
        PhysicsHandBone::LittleTip => return None,
    };
}

fn get_hand_res(res: &Res<'_, HandsResource>, hand: Hand) -> HandResource {
    match hand {
        Hand::Left => res.left.clone(),
        Hand::Right => res.right.clone(),
    }
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
    action_sets: Res<XrActionSets>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        //lock frame
        let frame_state = *frame_state.lock().unwrap();
        //get controller
        let controller = oculus_controller.get_ref(&session, &frame_state, &xr_input, &action_sets);
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
    action_sets: Res<XrActionSets>,
) {
    //lock frame
    let frame_state = *frame_state.lock().unwrap();
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

//this event is for transitioning the physics hand in an out of existent so we can drop things better
#[derive(Event)]
pub struct GhostHandEvent {
    pub hand: Hand,
    pub desired_state: bool, //true for no interactions, false for normal interactions
}
#[derive(Resource)]
pub struct GhostTimers {
    pub left: Timer,
    pub right: Timer,
}

pub fn handle_ghost_hand_events(
    mut events: EventReader<GhostHandEvent>,
    mut bones: Query<(&Hand, &mut CollisionGroups, With<PhysicsHandBone>)>,
) {
    for event in events.read() {
        // info!(
        //     "Ghost hand Event: {:?}, {:?}",
        //     event.hand, event.desired_state
        // );
        //do work
        for mut bone in bones.iter_mut() {
            match *bone.0 == event.hand {
                true => match event.desired_state {
                    true => bone.1.filters = Group::NONE,
                    false => bone.1.filters = Group::from_bits(0b0001).unwrap(),
                },
                false => (),
            }
        }
    }
}

pub fn watch_ghost_timers(
    mut timers: ResMut<GhostTimers>,
    mut writer: EventWriter<GhostHandEvent>,
    time: Res<Time>,
) {
    //tick both timers
    timers.left.tick(time.delta());
    timers.right.tick(time.delta());
    //if they finish send events to make the hands physical again
    if timers.left.just_finished() {
        writer.send(GhostHandEvent {
            hand: Hand::Left,
            desired_state: false,
        });
    }
    if timers.right.just_finished() {
        writer.send(GhostHandEvent {
            hand: Hand::Right,
            desired_state: false,
        });
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
        &Hand,
        Without<Grabbable>,
    )>,
    mut writer: EventWriter<GhostHandEvent>,
    mut timers: ResMut<GhostTimers>,
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
                                                //raise enter ghost hand event
                                                writer.send(GhostHandEvent {
                                                    hand: *interactor_transform.3,
                                                    desired_state: true,
                                                });
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
                                        *interactor_transform.2 = XRSelection::Empty;
                                        //reset timers to make hands physical again
                                        match *interactor_transform.3 {
                                            Hand::Left => timers.left.reset(),
                                            Hand::Right => timers.right.reset(),
                                        }
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
