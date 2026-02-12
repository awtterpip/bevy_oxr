use std::ops::Deref;

use bevy::prelude::*;
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    add_xr_plugins, openxr_session_running,
    resources::OxrInstance,
    session::OxrSession,
    spaces::OxrSpaceExt,
};
use bevy_mod_xr::{
    session::{session_available, XrSessionCreated},
    spaces::XrSpace,
};
use openxr::Posef;

fn main() -> AppExit {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins));
    app.add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin);
    app.add_systems(XrSessionCreated, spawn_hands);
    app.add_systems(XrSessionCreated, attach_set);
    app.add_systems(
        PreUpdate,
        sync_actions
            .before(OxrActionSetSyncSet)
            .run_if(openxr_session_running),
    );
    app.add_systems(OxrSendActionBindings, suggest_action_bindings);
    app.add_systems(Startup, create_actions.run_if(session_available));
    app.add_systems(Startup, setup);

    app.run()
}

fn attach_set(actions: Res<ControllerActions>, mut attach: MessageWriter<OxrAttachActionSet>) {
    attach.write(OxrAttachActionSet(actions.set.clone()));
}

#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left: openxr::Action<Posef>,
    right: openxr::Action<Posef>,
}
fn sync_actions(actions: Res<ControllerActions>, mut sync: MessageWriter<OxrSyncActionSet>) {
    sync.write(OxrSyncActionSet(actions.set.clone()));
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
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
fn suggest_action_bindings(
    actions: Res<ControllerActions>,
    mut bindings: MessageWriter<OxrSuggestActionBinding>,
) {
    bindings.write(OxrSuggestActionBinding {
        action: actions.left.as_raw(),
        interaction_profile: "/interaction_profiles/oculus/touch_controller".into(),
        bindings: vec!["/user/hand/left/input/grip/pose".into()],
    });
    bindings.write(OxrSuggestActionBinding {
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
    session: Res<OxrSession>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // This is a demonstation of how to integrate with the openxr crate, the right space is the
    // recommended way
    let left_space = XrSpace::from_openxr_space(
        actions
            .left
            .create_space(
                session.deref().deref(),
                openxr::Path::NULL,
                Posef::IDENTITY,
            )
            .unwrap(),
    );
    let right_space = session
        .create_action_space(&actions.right, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        left_space,
        Controller,
    ));
    cmds.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.1, 0.1, 0.05))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        right_space,
        Controller,
    ));
}

#[derive(Component)]
struct Controller;
