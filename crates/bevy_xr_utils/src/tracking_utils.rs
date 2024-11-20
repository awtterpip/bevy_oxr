use bevy::prelude::*;
use bevy_mod_openxr::{
    action_binding::{OxrSendActionBindings, OxrSuggestActionBinding},
    action_set_attaching::OxrAttachActionSet,
    action_set_syncing::{OxrActionSetSyncSet, OxrSyncActionSet},
    helper_traits::{ToQuat, ToVec3},
    openxr_session_available, openxr_session_running,
    resources::{OxrFrameState, OxrInstance, Pipelined},
    session::OxrSession,
    spaces::{OxrSpaceLocationFlags, OxrSpaceSyncSet},
};
use bevy_mod_xr::{
    session::{XrSessionCreated, XrTracker, XrTrackingRoot},
    spaces::{XrPrimaryReferenceSpace, XrReferenceSpace},
};
use openxr::Posef;

//exernal api
#[derive(Component)]
pub struct XrTrackedStage;

#[derive(Component)]
pub struct XrTrackedLocalFloor;

#[derive(Component)]
pub struct XrTrackedView;

#[derive(Component)]
pub struct XrTrackedLeftGrip;

#[derive(Component)]
pub struct XrTrackedRightGrip;

pub struct TrackingUtilitiesPlugin;

impl Plugin for TrackingUtilitiesPlugin {
    fn build(&self, app: &mut App) {
        //spawn tracking rig
        app.add_systems(XrSessionCreated, spawn_tracking_rig);

        //update stage transforms
        //external
        app.add_systems(PreUpdate, update_stage);

        //head view transforms
        //internal
        app.add_systems(
            PreUpdate,
            update_head_transforms
                .in_set(OxrSpaceSyncSet)
                .run_if(openxr_session_running),
        );
        //external
        app.add_systems(PreUpdate, update_view.after(update_head_transforms));

        //local floor transforms
        app.add_systems(
            PreUpdate,
            update_local_floor_transforms.after(update_head_transforms),
        );

        //bindings
        app.add_systems(OxrSendActionBindings, suggest_action_bindings);
        //sync actions
        app.add_systems(
            PreUpdate,
            sync_actions
                .before(OxrActionSetSyncSet)
                .run_if(openxr_session_running),
        );
        //attach sets
        app.add_systems(XrSessionCreated, attach_set);
        //create actions
        app.add_systems(Startup, create_actions.run_if(openxr_session_available));

        app.add_systems(PreUpdate, update_left_grip.after(OxrSpaceSyncSet));
        app.add_systems(PreUpdate, update_right_grip.after(OxrSpaceSyncSet));
    }
}

//stage
fn update_stage(
    root_query: Query<&Transform, (With<XrTrackingRoot>, Without<XrTrackedStage>)>,
    mut stage_query: Query<&mut Transform, (With<XrTrackedStage>, Without<XrTrackingRoot>)>,
) {
    if let Ok(root) = root_query.get_single() {
        for mut transform in &mut stage_query {
            *transform = *root;
        }
    }
}

//view
#[derive(Component)]
struct HeadXRSpace(XrReferenceSpace);

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

fn update_view(
    mut head_query: Query<&mut Transform, (With<HeadXRSpace>, Without<XrTrackedView>)>,
    mut view_query: Query<&mut Transform, (With<XrTrackedView>, Without<HeadXRSpace>)>,
) {
    let head_transform = head_query.get_single_mut();
    if let Ok(root) = head_transform {
        for mut transform in &mut view_query {
            *transform = *root;
        }
    }
}

//local floor
fn update_local_floor_transforms(
    mut head_space: Query<&mut Transform, (With<HeadXRSpace>, Without<XrTrackedLocalFloor>)>,
    mut local_floor: Query<&mut Transform, (With<XrTrackedLocalFloor>, Without<HeadXRSpace>)>,
) {
    let head_transform = head_space.get_single_mut();
    if let Ok(head) = head_transform {
        let mut calc_floor = *head;
        calc_floor.translation.y = 0.0;
        //TODO: use yaw
        let (y, _, _) = calc_floor.rotation.to_euler(EulerRot::YXZ);
        let new_rot = Quat::from_rotation_y(y);
        calc_floor.rotation = new_rot;
        for mut transform in &mut local_floor {
            *transform = calc_floor;
        }
    }
}

//left grip
#[derive(Component)]
struct LeftGrip;

fn update_left_grip(
    mut left_grip: Query<&mut Transform, (With<LeftGrip>, Without<XrTrackedLeftGrip>)>,
    mut tracked_left_grip: Query<&mut Transform, (With<XrTrackedLeftGrip>, Without<LeftGrip>)>,
) {
    let head_transform = left_grip.get_single_mut();
    if let Ok(head) = head_transform {
        for mut transform in &mut tracked_left_grip {
            *transform = *head;
        }
    }
}

//right grip
#[derive(Component)]
struct RightGrip;

fn update_right_grip(
    mut right_grip: Query<&mut Transform, (With<RightGrip>, Without<XrTrackedRightGrip>)>,
    mut tracked_right_grip: Query<&mut Transform, (With<XrTrackedRightGrip>, Without<RightGrip>)>,
) {
    let head_transform = right_grip.get_single_mut();
    if let Ok(head) = head_transform {
        for mut transform in &mut tracked_right_grip {
            *transform = *head;
        }
    }
}

//tracking rig
#[derive(Resource)]
struct ControllerActions {
    set: openxr::ActionSet,
    left: openxr::Action<Posef>,
    right: openxr::Action<Posef>,
}

fn spawn_tracking_rig(
    actions: Res<ControllerActions>,
    mut cmds: Commands,
    session: Res<OxrSession>,
) {
    //head
    let head_space = session
        .create_reference_space(openxr::ReferenceSpaceType::VIEW, Transform::IDENTITY)
        .unwrap();
    cmds.spawn((
        Transform::default(),
        Visibility::default(),
        XrTracker,
        HeadXRSpace(head_space),
    ));
    // let local_floor = cmds.spawn((SpatialBundle::default(), LocalFloor)).id();

    let left_space = session
        .create_action_space(&actions.left, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    let right_space = session
        .create_action_space(&actions.right, openxr::Path::NULL, Isometry3d::IDENTITY)
        .unwrap();
    cmds.spawn((left_space, LeftGrip));
    cmds.spawn((right_space, RightGrip));
}

//bindings
//TODO figure out how to make these better, specifically not be controller specific
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
