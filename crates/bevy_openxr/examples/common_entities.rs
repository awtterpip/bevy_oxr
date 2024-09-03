//! A simple 3D scene with light shining over a cube sitting on a plane.

use std::ops::Deref;

use bevy::prelude::*;
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    add_xr_plugins,
    helper_traits::{ToQuat, ToVec3},
    resources::{OxrFrameState, OxrInstance, Pipelined},
    session::OxrSession,
    spaces::{OxrSpaceExt, OxrSpaceLocationFlags, OxrSpaceSyncSet, OxrSpaceVelocityFlags},
};
use bevy_mod_xr::{
    session::{session_available, session_running, XrSessionCreated, XrTrackingRoot},
    spaces::{XrPrimaryReferenceSpace, XrReferenceSpace, XrSpace, XrVelocity},
    types::XrPose,
};
use openxr::Posef;

fn main() {
    let mut app = App::new();
    app.add_plugins(add_xr_plugins(DefaultPlugins));
    app.add_systems(Startup, setup);
    //create bindings
    app.add_systems(OxrSendActionBindings, suggest_action_bindings);
    //sync actions
    app.add_systems(
        PreUpdate,
        sync_actions
            .before(OxrActionSetSyncSet)
            .run_if(session_running),
    );
    //things?
    app.add_systems(XrSessionCreated, spawn_hands);
    app.add_systems(XrSessionCreated, attach_set);
    app.add_systems(Startup, create_actions.run_if(session_available));

    //head space
    app.add_systems(
        PreUpdate,
        update_head_transforms
            .in_set(OxrSpaceSyncSet)
            .run_if(session_running),
    );
    //local floor emulated
    app.add_systems(PreUpdate, update_local_floor.after(update_head_transforms));

    app.run();
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
        material: materials.add(Color::srgb_u8(124, 144, 255)),
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

#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left: openxr::Action<Posef>,
    right: openxr::Action<Posef>,
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

fn sync_actions(actions: Res<ControllerActions>, mut sync: EventWriter<OxrSyncActionSet>) {
    sync.send(OxrSyncActionSet(actions.set.clone()));
}

fn attach_set(actions: Res<ControllerActions>, mut attach: EventWriter<OxrAttachActionSet>) {
    attach.send(OxrAttachActionSet(actions.set.clone()));
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
    root: Query<Entity, With<XrTrackingRoot>>,
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
                session.deref().deref().clone(),
                openxr::Path::NULL,
                Posef::IDENTITY,
            )
            .unwrap(),
    );
    let right_space = session
        .create_action_space(&actions.right, openxr::Path::NULL, XrPose::IDENTITY)
        .unwrap();
    let left = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.05)),
                material: materials.add(Color::srgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            left_space,
        ))
        .id();
    let right = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.1, 0.1, 0.05)),
                material: materials.add(Color::srgb_u8(124, 144, 255)),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..default()
            },
            right_space,
        ))
        .id();
    //head?
    let head_space = session
        .create_reference_space(openxr::ReferenceSpaceType::VIEW, Transform::IDENTITY)
        .unwrap();
    let head = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.2, 0.2, 0.2)),
                material: materials.add(Color::srgb_u8(255, 144, 144)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            HeadXRSpace(head_space),
        ))
        .id();
    //local_floor? emulated
    let local_floor = cmds
        .spawn((
            PbrBundle {
                mesh: meshes.add(Cuboid::new(0.5, 0.1, 0.5)),
                material: materials.add(Color::srgb_u8(144, 255, 144)),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            },
            LocalFloor,
        ))
        .id();

    cmds.entity(root.single())
        .push_children(&[left, right, head, local_floor]);
}

#[derive(Component)]
struct HeadXRSpace(XrReferenceSpace);

#[allow(clippy::type_complexity)]
fn update_head_transforms(
    session: Res<OxrSession>,
    default_ref_space: Res<XrPrimaryReferenceSpace>,
    pipelined: Option<Res<Pipelined>>,
    frame_state: Res<OxrFrameState>,
    mut query: Query<(&mut Transform, &HeadXRSpace, Option<&XrReferenceSpace>)>,
) {
    for (mut transform, space, ref_space) in &mut query {
        let ref_space = ref_space.unwrap_or(&default_ref_space);
        let time = if pipelined.is_some() {
            openxr::Time::from_nanos(
                frame_state.predicted_display_time.as_nanos()
                    + frame_state.predicted_display_period.as_nanos(),
            )
        } else {
            frame_state.predicted_display_time
        };
        let space_location = session.locate_space(&space.0, ref_space, time);

        if let Ok(space_location) = space_location {
            let flags = OxrSpaceLocationFlags(space_location.location_flags);
            if flags.pos_valid() {
                transform.translation = space_location.pose.position.to_vec3();
            }
            if flags.rot_valid() {
                transform.rotation = space_location.pose.orientation.to_quat();
            }
        }
    }
}

//emulated local_floor
#[derive(Component)]
struct LocalFloor;

fn update_local_floor(
    mut headSpace: Query<&mut Transform, (With<HeadXRSpace>, Without<LocalFloor>)>,
    mut local_floor: Query<&mut Transform, (With<LocalFloor>, Without<HeadXRSpace>)>,
) {
    let head_transform = headSpace.get_single_mut();
    match head_transform {
        Ok(head) => {
            let mut calc_floor = head.clone();
            calc_floor.translation.y = 0.0;
            calc_floor.rotation = Quat::IDENTITY;
            for (mut transform) in &mut local_floor {
                *transform = calc_floor;
            }
        }
        Err(_) => (),
    }
}
