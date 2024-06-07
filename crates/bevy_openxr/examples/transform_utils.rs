//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_openxr::add_xr_plugins;
use bevy_xr_utils::xr_utils_actions::{ActiveSet, XRUtilsAction, XRUtilsActionSet, XRUtilsActionState, XRUtilsActionSystemSet, XRUtilsActionsPlugin, XRUtilsBinding};
use bevy_xr_utils::transform_utils;

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_plugins(transform_utils::TransformUtilitiesPlugin)
        .add_systems(Startup, setup)
        .add_plugins(XRUtilsActionsPlugin)
        .add_systems(
            Startup,
            create_action_entities.before(XRUtilsActionSystemSet::CreateEvents),
        )
        .add_systems(
            Update,
            read_action_with_marker_component.after(XRUtilsActionSystemSet::SyncActionStates),
        )
        .run();
}

/// set up a simple 3D scene
fn setup(
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
    // red cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(252, 44, 3)),
        transform: Transform::from_xyz(4.0, 0.5, 0.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // red cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(3, 28, 252)),
        transform: Transform::from_xyz(-4.0, 0.5, 0.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // green cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(3, 252, 32)),
        transform: Transform::from_xyz(0.0, 0.5, 4.0).with_scale(Vec3::splat(0.5)),
        ..default()
    });
    // white cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::rgb_u8(250, 250, 250)),
        transform: Transform::from_xyz(0.0, 0.5, -4.0).with_scale(Vec3::splat(0.5)),
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
pub struct FaceRedAction;

fn create_action_entities(mut commands: Commands) {
    //create a set
    let set = commands
        .spawn((
            XRUtilsActionSet {
                name: "locomotion".into(),
                pretty_name: "locomotion set".into(),
                priority: u32::MIN,
            },
            ActiveSet, //marker to indicate we want this synced
        ))
        .id();
    //create an action
    let action = commands
        .spawn((
            XRUtilsAction {
                action_name: "face_red".into(),
                localized_name: "face_red_localized".into(),
                action_type: bevy_xr::actions::ActionType::Bool,
            },
            FaceRedAction, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding = commands
        .spawn(XRUtilsBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/left//input/a/click".into(),
        })
        .id();

    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding);
    commands.entity(set).add_child(action);
}

fn read_action_with_marker_component(
    mut action_query: Query<&XRUtilsActionState, With<FaceRedAction>>,
) {
    //now for the actual checking
    for state in action_query.iter_mut() {
        info!("action state is: {:?}", state);
    }
}
