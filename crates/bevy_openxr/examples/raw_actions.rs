use std::ops::Deref;

use bevy::prelude::*;
use bevy_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    add_xr_plugins,
    init::OxrTrackingRoot,
    resources::OxrInstance,
    session::OxrSession,
    spaces::OxrSpaceExt,
};
use bevy_xr::{
    session::{session_available, XrSessionCreated},
    spaces::{XrSpace, XrSpatialTransform},
    types::XrPose,
};
use openxr::Posef;

fn main() {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins));
    app.add_systems(XrSessionCreated, spawn_hands);
    app.add_systems(XrSessionCreated, attach_set);
    app.add_systems(PreUpdate, sync_actions.before(OxrActionSetSyncSet));
    app.add_systems(OxrSendActionBindings, suggest_action_bindings);
    app.add_systems(Startup, create_actions.run_if(session_available));
    app.add_systems(Startup, setup);

    app.run();
}

fn attach_set(actions: Res<ControllerActions>, mut attach: EventWriter<OxrAttachActionSet>) {
    attach.send(OxrAttachActionSet(actions.set.clone()));
}

#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left: openxr::Action<Posef>,
    right: openxr::Action<Posef>,
}
fn sync_actions(actions: Res<ControllerActions>, mut sync: EventWriter<OxrSyncActionSet>) {
    sync.send(OxrSyncActionSet(actions.set.clone()));
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
fn suggest_action_bindings(
    actions: Res<ControllerActions>,
    mut bindings: EventWriter<OxrSuggestActionBinding>,
) {
    bindings.send(OxrSuggestActionBinding {
        action: actions.left.as_raw(),
        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
        bindings: vec!["/user/hand/left/input/grip/pose".into()],
    });
    bindings.send(OxrSuggestActionBinding {
        action: actions.right.as_raw(),
        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
        bindings: vec!["/user/hand/right/input/grip/pose".into()],
    });
}
fn create_actions(instance: Res<OxrInstance>, mut cmds: Commands) {
    let set = instance.create_action_set("hands", "Hands", 0).unwrap();
    let left = set
        .create_action("left_pose", "Left Hand Grip Pose", &[])
        .unwrap();
    let right = set
        .create_action("right_pose", "Right Hand Grip Pose", &[])
        .unwrap();

    cmds.insert_resource(ControllerActions { set, left, right })
}

fn spawn_hands(
    actions: Res<ControllerActions>,
    mut cmds: Commands,
    root: Query<Entity, With<OxrTrackingRoot>>,
    session: Res<OxrSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let l = actions
        .left
        .create_space(
            session.deref().deref().clone(),
            openxr::Path::NULL,
            Posef::IDENTITY,
        )
        .unwrap();
    let left_space = XrSpace::from_openxr_space(l);
    // let left_space = session
    //     .create_action_space(&actions.left, openxr::Path::NULL, XrPose::IDENTITY)
    //     .unwrap();
    let right_space = session
        .create_action_space(&actions.right, openxr::Path::NULL, XrPose::IDENTITY)
        .unwrap();
    let left = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.05)),
                material: materials.add(Color::rgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            XrSpatialTransform::from_space(left_space),
            Controller,
        ))
        .id();
    let right = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.05)),
                material: materials.add(Color::rgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            XrSpatialTransform::from_space(right_space),
            Controller,
        ))
        .id();

    cmds.entity(root.single()).push_children(&[left, right]);
}

#[derive(Component)]
struct Controller;
