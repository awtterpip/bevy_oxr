use bevy::prelude::*;
use bevy_mod_openxr::{
    helper_traits::{ToQuat, ToVec3},
    resources::{OxrFrameState, Pipelined},
    session::OxrSession,
    spaces::{OxrSpaceLocationFlags, OxrSpaceSyncSet},
};
use bevy_mod_xr::{
    session::{session_running, XrSessionCreated, XrTrackingRoot},
    spaces::{XrPrimaryReferenceSpace, XrReferenceSpace},
};
use openxr::Posef;

//exernal api
#[derive(Component)]
pub struct XRTrackedStage;

#[derive(Component)]
pub struct XRTrackedLocalFloor;

#[derive(Component)]
pub struct XRTrackedView;

#[derive(Component)]
pub struct XRTrackedLeftGrip;

#[derive(Component)]
pub struct XRTRackedRightGrip;

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
                .run_if(session_running),
        );
        //external
        app.add_systems(PreUpdate, update_view.after(update_head_transforms));

        //local floor transforms
        //internal
        app.add_systems(
            PreUpdate,
            update_local_floor_transforms.after(update_head_transforms),
        );
        //external
        app.add_systems(
            PreUpdate,
            update_local_floor.after(update_local_floor_transforms),
        );
    }
}

//stage
fn update_stage(
    mut root_query: Query<&mut Transform, (With<XrTrackingRoot>, Without<XRTrackedStage>)>,
    mut stage_query: Query<&mut Transform, (With<XRTrackedStage>, Without<XrTrackingRoot>)>,
) {
    let tracking_root_transform = root_query.get_single_mut();
    match tracking_root_transform {
        Ok(root) => {
            for (mut transform) in &mut stage_query {
                *transform = root.clone();
            }
        }
        Err(_) => (),
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
    mut head_query: Query<&mut Transform, (With<HeadXRSpace>, Without<XRTrackedView>)>,
    mut view_query: Query<&mut Transform, (With<XRTrackedView>, Without<HeadXRSpace>)>,
) {
    let head_transform = head_query.get_single_mut();
    match head_transform {
        Ok(root) => {
            for (mut transform) in &mut view_query {
                *transform = root.clone();
            }
        }
        Err(_) => (),
    }
}

//local floor
#[derive(Component)]
struct LocalFloor;
//internal
fn update_local_floor_transforms(
    mut head_space: Query<&mut Transform, (With<HeadXRSpace>, Without<LocalFloor>)>,
    mut local_floor: Query<&mut Transform, (With<LocalFloor>, Without<HeadXRSpace>)>,
) {
    let head_transform = head_space.get_single_mut();
    match head_transform {
        Ok(head) => {
            let mut calc_floor = head.clone();
            calc_floor.translation.y = 0.0;
            //TODO: use yaw
            calc_floor.rotation = Quat::IDENTITY;
            for (mut transform) in &mut local_floor {
                *transform = calc_floor;
            }
        }
        Err(_) => (),
    }
}
//external
fn update_local_floor(
    mut local_floor: Query<&mut Transform, (With<LocalFloor>, Without<XRTrackedLocalFloor>)>,
    mut tracked_floor: Query<&mut Transform, (With<XRTrackedLocalFloor>, Without<LocalFloor>)>,
) {
    let head_transform = local_floor.get_single_mut();
    match head_transform {
        Ok(head) => {
            for (mut transform) in &mut tracked_floor {
                *transform = head.clone();
            }
        }
        Err(_) => (),
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
    // actions: Res<ControllerActions>,
    mut cmds: Commands,
    // root: Query<Entity, With<XrTrackingRoot>>,
    session: Res<OxrSession>,
) {
    //head
    let head_space = session
        .create_reference_space(openxr::ReferenceSpaceType::VIEW, Transform::IDENTITY)
        .unwrap();
    let head = cmds
        .spawn((SpatialBundle { ..default() }, HeadXRSpace(head_space)))
        .id();
    let local_floor = cmds.spawn((SpatialBundle { ..default() }, LocalFloor)).id();
}
