// a simple example showing basic actions

use std::borrow::Cow;

use bevy::prelude::*;
use bevy_openxr::{
    action_binding::OxrSuggestActionBinding,
    action_set_attaching::{AttachedActionSets, OxrAttachActionSet},
    add_xr_plugins,
    resources::{OxrInstance, OxrSession},
};
use openxr::{ActiveActionSet, Path, Vector2f};

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup_scene)
        .add_systems(Startup, create_action_entities)
        .add_systems(Startup, create_openxr_events.after(create_action_entities))
        .add_systems(Update, sync_actions)
        .add_systems(
            Update,
            sync_and_update_action_states_f32.after(sync_actions),
        )
        .add_systems(
            Update,
            sync_and_update_action_states_bool.after(sync_actions),
        )
        .add_systems(Update, read_action_with_marker_component.after(sync_and_update_action_states_f32))
        .run();
}

/// set up a simple 3D scene
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

#[derive(Component)]
struct CreateActionSet {
    name: Cow<'static, str>,
    pretty_name: Cow<'static, str>,
    priority: u32,
}

#[derive(Component)]
struct CreateAction {
    action_name: Cow<'static, str>,
    localized_name: Cow<'static, str>,
    action_type: bevy_xr::actions::ActionType,
}

#[derive(Component)]
struct CreateBinding {
    profile: Cow<'static, str>,
    binding: Cow<'static, str>,
}

#[derive(Component)]
struct Actionf32Reference {
    action: openxr::Action<f32>,
}

#[derive(Component)]
struct ActionBooleference {
    action: openxr::Action<bool>,
}

#[derive(Component)]
struct ActionVector2fReference {
    action: openxr::Action<Vector2f>,
}

#[derive(Component)]
struct CustomActionMarker;

fn create_action_entities(mut commands: Commands) {
    //create a set
    let set = commands
        .spawn(CreateActionSet {
            name: "test".into(),
            pretty_name: "pretty test".into(),
            priority: u32::MIN,
        })
        .id();
    //create an action
    let action = commands
        .spawn((
            CreateAction {
                action_name: "action_name".into(),
                localized_name: "localized_name".into(),
                action_type: bevy_xr::actions::ActionType::Float,
            },
            CustomActionMarker, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding = commands
        .spawn(CreateBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/right/input/thumbstick/y".into(),
        })
        .id();

    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding);
    commands.entity(set).add_child(action);

    //create an action
    let action = commands
        .spawn((
            CreateAction {
                action_name: "action_name_bool".into(),
                localized_name: "localized_name_bool".into(),
                action_type: bevy_xr::actions::ActionType::Bool,
            },
            CustomActionMarker, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding = commands
        .spawn(CreateBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/right/input/a/click".into(),
        })
        .id();

    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding);
    commands.entity(set).add_child(action);
}

fn create_openxr_events(
    action_sets_query: Query<(&CreateActionSet, &Children)>,
    actions_query: Query<(&CreateAction, &Children)>,
    bindings_query: Query<&CreateBinding>,
    instance: ResMut<OxrInstance>,
    mut binding_writer: EventWriter<OxrSuggestActionBinding>,
    mut attach_writer: EventWriter<OxrAttachActionSet>,
    //not my favorite way of doing this
    mut commands: Commands,
) {
    //lets create some sets!
    //we gonna need a collection of these sets for later
    // let mut ActionSets = HashMap::new();
    for (set, children) in action_sets_query.iter() {
        //create action set
        let action_set: openxr::ActionSet = instance
            .create_action_set(&set.name, &set.pretty_name, set.priority)
            .unwrap();

        //since the actions are made from the sets lets go
        for &child in children.iter() {
            //first get the action entity and stuff
            let (create_action, bindings) = actions_query.get(child).unwrap();
            //lets create dat actions
            match create_action.action_type {
                bevy_xr::actions::ActionType::Bool => {
                    let action: openxr::Action<bool> = action_set
                        .create_action::<bool>(
                            &create_action.action_name,
                            &create_action.localized_name,
                            &[],
                        )
                        .unwrap();
                    //please put this in a function so I dont go crazy
                    //insert a reference for later
                    commands.entity(child).insert((
                        ActionBooleference {
                            action: action.clone(),
                        },
                        MyActionState::Bool(ActionStateBool {
                            current_state: false,
                            changed_since_last_sync: false,
                            last_change_time: i64::MIN,
                            is_active: false,
                        }),
                    ));
                    //since we need actions for bindings lets go!!
                    for &bind in bindings.iter() {
                        //interaction profile
                        //get the binding entity and stuff
                        let create_binding = bindings_query.get(bind).unwrap();
                        let profile = Cow::from(create_binding.profile.clone());
                        //bindings
                        let binding = vec![Cow::from(create_binding.binding.clone())];
                        let sugestion = OxrSuggestActionBinding {
                            action: action.as_raw(),
                            interaction_profile: profile,
                            bindings: binding,
                        };
                        //finally send the suggestion
                        binding_writer.send(sugestion);
                    }
                }
                bevy_xr::actions::ActionType::Float => {
                    let action: openxr::Action<f32> = action_set
                        .create_action::<f32>(
                            &create_action.action_name,
                            &create_action.localized_name,
                            &[],
                        )
                        .unwrap();

                    //please put this in a function so I dont go crazy
                    //insert a reference for later
                    commands.entity(child).insert((
                        Actionf32Reference {
                            action: action.clone(),
                        },
                        MyActionState::Float(ActionStateFloat {
                            current_state: 0.0,
                            changed_since_last_sync: false,
                            last_change_time: i64::MIN,
                            is_active: false,
                        }),
                    ));
                    //since we need actions for bindings lets go!!
                    for &bind in bindings.iter() {
                        //interaction profile
                        //get the binding entity and stuff
                        let create_binding = bindings_query.get(bind).unwrap();
                        let profile = Cow::from(create_binding.profile.clone());
                        //bindings
                        let binding = vec![Cow::from(create_binding.binding.clone())];
                        let sugestion = OxrSuggestActionBinding {
                            action: action.as_raw(),
                            interaction_profile: profile,
                            bindings: binding,
                        };
                        //finally send the suggestion
                        binding_writer.send(sugestion);
                    }
                }
                bevy_xr::actions::ActionType::Vector => {
                    let action: openxr::Action<Vector2f> = action_set
                        .create_action::<Vector2f>(
                            &create_action.action_name,
                            &create_action.localized_name,
                            &[],
                        )
                        .unwrap();

                    //please put this in a function so I dont go crazy
                    //insert a reference for later
                    commands.entity(child).insert((
                        ActionVector2fReference {
                            action: action.clone(),
                        },
                        MyActionState::Vector(ActionStateVector {
                            current_state: [0.0, 0.0],
                            changed_since_last_sync: false,
                            last_change_time: i64::MIN,
                            is_active: false,
                        }),
                    ));
                    //since we need actions for bindings lets go!!
                    for &bind in bindings.iter() {
                        //interaction profile
                        //get the binding entity and stuff
                        let create_binding = bindings_query.get(bind).unwrap();
                        let profile = Cow::from(create_binding.profile.clone());
                        //bindings
                        let binding = vec![Cow::from(create_binding.binding.clone())];
                        let sugestion = OxrSuggestActionBinding {
                            action: action.as_raw(),
                            interaction_profile: profile,
                            bindings: binding,
                        };
                        //finally send the suggestion
                        binding_writer.send(sugestion);
                    }
                }
            };
        }

        attach_writer.send(OxrAttachActionSet(action_set));
    }
}

fn sync_actions(session: Res<OxrSession>, attached: Res<AttachedActionSets>) {
    //first we need to sync our actions
    let why = &attached
        .sets
        .iter()
        .map(|v| ActiveActionSet::from(v))
        .collect::<Vec<_>>();
    let sync = session.sync_actions(&why[..]);
    match sync {
        Ok(_) => info!("sync ok"),
        Err(_) => error!("sync error"),
    }
}

fn sync_and_update_action_states_f32(
    session: Res<OxrSession>,
    mut f32_query: Query<(&Actionf32Reference, &mut MyActionState)>,
) {
    //now we do the action state for f32
    for (reference, mut silly_state) in f32_query.iter_mut() {
        let state = reference.action.state(&session, Path::NULL);
        match state {
            Ok(s) => {
                info!("we found a state");
                let new_state = MyActionState::Float(ActionStateFloat {
                    current_state: s.current_state,
                    changed_since_last_sync: s.changed_since_last_sync,
                    last_change_time: s.last_change_time.as_nanos(),
                    is_active: s.is_active,
                });

                *silly_state = new_state;
            }
            Err(_) => {
                info!("error getting action state");
            }
        }
    }
}

fn sync_and_update_action_states_bool(
    session: Res<OxrSession>,
    mut f32_query: Query<(&ActionBooleference, &mut MyActionState)>,
) {
    //now we do the action state for f32
    for (reference, mut silly_state) in f32_query.iter_mut() {
        let state = reference.action.state(&session, Path::NULL);
        match state {
            Ok(s) => {
                info!("we found a state");
                let new_state = MyActionState::Bool(ActionStateBool {
                    current_state: s.current_state,
                    changed_since_last_sync: s.changed_since_last_sync,
                    last_change_time: s.last_change_time.as_nanos(),
                    is_active: s.is_active,
                });

                *silly_state = new_state;
            }
            Err(_) => {
                info!("error getting action state");
            }
        }
    }
}

fn read_action_with_marker_component(
    mut action_query: Query<&MyActionState, With<CustomActionMarker>>,
) {
    //now for the actual checking
    for state in action_query.iter_mut() {
        info!("action state is: {:?}", state);
    }
}

//the things i do for bad prototyping and lack of understanding
#[derive(Component, Debug)]
pub enum MyActionState {
    Bool(ActionStateBool),
    Float(ActionStateFloat),
    Vector(ActionStateVector),
}

#[derive(Debug)]
pub struct ActionStateBool {
    pub current_state: bool,
    pub changed_since_last_sync: bool,
    pub last_change_time: i64,
    pub is_active: bool,
}
#[derive(Debug)]
pub struct ActionStateFloat {
    pub current_state: f32,
    pub changed_since_last_sync: bool,
    pub last_change_time: i64,
    pub is_active: bool,
}
#[derive(Debug)]
pub struct ActionStateVector {
    pub current_state: [f32; 2],
    pub changed_since_last_sync: bool,
    pub last_change_time: i64,
    pub is_active: bool,
}
