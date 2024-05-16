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
        .add_systems(Update, read_action_with_marker_component)
        .add_systems(Startup, create_action_entities)
        .add_systems(Startup, create_openxr_events.after(create_action_entities))
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
struct ActionReference {
    action: openxr::Action<f32>,
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
                    //TODO remove this crap
                    // test.action = Some(action.clone());
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
                    commands.entity(child).insert(ActionReference {
                        action: action.clone(),
                    });
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
                    //TODO remove this crap
                    // test.action = Some(action.clone());
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

fn read_action_with_marker_component(
    action_query: Query<&ActionReference, With<CustomActionMarker>>,
    session: ResMut<OxrSession>,
    attached: ResMut<AttachedActionSets>,
) {
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
    
    //now for the actual checking
    let action = action_query.get_single();
    match action {
        Ok(reference) => {
            let state = reference.action.state(&session, Path::NULL);
            match state {
                Ok(a) => {
                    info!("action state: {:?}", a);
                }
                Err(_) => info!("error getting state"),
            }
        }
        Err(_) => {
            info!("no action")
        }
    }
}
