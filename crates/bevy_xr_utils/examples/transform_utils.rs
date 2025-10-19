//! A simple example of how to use the transform utils to set the players position and orientation

use bevy::prelude::*;
use bevy_mod_openxr::add_xr_plugins;
use bevy_xr_utils::transform_utils::{self, SnapToPosition, SnapToRotation};
use bevy_xr_utils::xr_utils_actions::{
    ActiveSet, XRUtilsAction, XRUtilsActionSet, XRUtilsActionState, XRUtilsActionSystems,
    XRUtilsActionsPlugin, XRUtilsBinding,
};

fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins))
        .add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin)
        .add_plugins(transform_utils::TransformUtilitiesPlugin)
        .add_systems(Startup, setup)
        .add_plugins(XRUtilsActionsPlugin)
        .add_systems(
            Startup,
            create_action_entities.before(XRUtilsActionSystems::CreateEvents),
        )
        .add_systems(
            Update,
            send_look_at_red_cube_event.after(XRUtilsActionSystems::SyncActionStates),
        )
        .add_systems(
            Update,
            send_recenter.after(XRUtilsActionSystems::SyncActionStates),
        )
        .insert_resource(AmbientLight {
            brightness: 500.0,
            ..AmbientLight::default()
        })
        .run()
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // red cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(252, 44, 3))),
        Transform::from_xyz(4.0, 0.5, 0.0).with_scale(Vec3::splat(0.5)),
    ));
    // blue cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(3, 28, 252))),
        Transform::from_xyz(-4.0, 0.5, 0.0).with_scale(Vec3::splat(0.5)),
    ));
    // green cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(3, 252, 32))),
        Transform::from_xyz(0.0, 0.5, 4.0).with_scale(Vec3::splat(0.5)),
    ));
    // white cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(250, 250, 250))),
        Transform::from_xyz(0.0, 0.5, -4.0).with_scale(Vec3::splat(0.5)),
    ));
    // black cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(0, 0, 0))),
        Transform::from_xyz(0.0, 0.1, 0.0).with_scale(Vec3::splat(0.2)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
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
                action_type: bevy_mod_xr::actions::ActionType::Bool,
            },
            FaceRedAction, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding = commands
        .spawn(XRUtilsBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/left/input/a/click".into(),
        })
        .id();

    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding);
    commands.entity(set).add_child(action);

    let action = commands
        .spawn((
            XRUtilsAction {
                action_name: "center".into(),
                localized_name: "center_localized".into(),
                action_type: bevy_mod_xr::actions::ActionType::Bool,
            },
            Center, //lets try a marker component
        ))
        .id();

    //create a binding
    let binding = commands
        .spawn(XRUtilsBinding {
            profile: "/interaction_profiles/valve/index_controller".into(),
            binding: "/user/hand/right/input/a/click".into(),
        })
        .id();

    //add action to set, this isnt the best
    //TODO look into a better system
    commands.entity(action).add_child(binding);
    commands.entity(set).add_child(action);
}

fn send_look_at_red_cube_event(
    mut action_query: Query<&XRUtilsActionState, With<FaceRedAction>>,
    mut event_writer: EventWriter<SnapToRotation>,
) {
    //now for the actual checking
    for state in action_query.iter_mut() {
        match state {
            XRUtilsActionState::Bool(state) => {
                let send = state.current_state && state.changed_since_last_sync;
                if send {
                    info!("send facing");
                    let quat = Transform::default()
                        .looking_at(Transform::from_xyz(4.0, 0.0, 0.0).translation, Vec3::Y); //this is a transform facing the red cube from the center of the scene, you should use the HMD posision but I was lazy.
                    event_writer.write(SnapToRotation(quat.rotation));
                }
            }
            XRUtilsActionState::Float(_) => (),
            XRUtilsActionState::Vector(_) => (),
        }
    }
}

#[derive(Component)]
pub struct Center;

fn send_recenter(
    mut action_query: Query<&XRUtilsActionState, With<Center>>,
    mut event_writer: EventWriter<SnapToPosition>,
) {
    //now for the actual checking
    for state in action_query.iter_mut() {
        match state {
            XRUtilsActionState::Bool(state) => {
                let send = state.current_state && state.changed_since_last_sync;
                if send {
                    let center = Transform::default().translation;

                    event_writer.write(SnapToPosition(center));
                }
            }
            XRUtilsActionState::Float(_) => (),
            XRUtilsActionState::Vector(_) => (),
        }
    }
}
