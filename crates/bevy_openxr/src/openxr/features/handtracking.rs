use bevy::prelude::*;
use bevy_mod_xr::hands::{LeftHand, RightHand, XrHandBoneEntities, HAND_JOINT_COUNT};
use bevy_mod_xr::session::{XrPreDestroySession, XrSessionCreated, XrTrackingRoot};
use bevy_mod_xr::spaces::{XrPrimaryReferenceSpace, XrReferenceSpace};
use bevy_mod_xr::{
    hands::{HandBone, HandBoneRadius},
    session::session_running,
};
use openxr::SpaceLocationFlags;

use crate::init::create_xr_session;
use crate::resources::OxrFrameState;
use crate::resources::Pipelined;
use crate::session::OxrSession;

pub struct HandTrackingPlugin {
    default_hands: bool,
}
impl Default for HandTrackingPlugin {
    fn default() -> Self {
        Self {
            default_hands: true,
        }
    }
}

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, locate_hands.run_if(session_running));
        if self.default_hands {
            app.add_systems(XrPreDestroySession, clean_up_default_hands)
                .add_systems(XrSessionCreated, spawn_default_hands);
        }
    }
}

pub fn spawn_hand_bones<T: Bundle + Clone>(
    cmds: &mut Commands,
    bundle: T,
) -> [Entity; HAND_JOINT_COUNT] {
    let mut bones: [Entity; HAND_JOINT_COUNT] = [Entity::PLACEHOLDER; HAND_JOINT_COUNT];
    // screw you clippy, i don't see a better way to init this array
    #[allow(clippy::needless_range_loop)]
    for bone in HandBone::get_all_bones().into_iter() {
        bones[bone as usize] = cmds
            .spawn((SpatialBundle::default(), bone, HandBoneRadius(0.0)))
            .insert(bundle.clone())
            .id();
    }
    bones
}

fn spawn_default_hands(
    mut cmds: Commands,
    session: Res<OxrSession>,
    root: Query<Entity, With<XrTrackingRoot>>,
) {
    debug!("spawning default hands");
    let Ok(root) = root.get_single() else {
        error!("unable to get tracking root, skipping hand creation");
        return;
    };
    let tracker_left = match session.create_hand_tracker(openxr::HandEXT::LEFT) {
        Ok(t) => t,
        Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
            warn!("Handtracking Extension not loaded, Unable to create Handtracker!");
            return;
        }
        Err(err) => {
            warn!("Error while creating Handtracker: {}", err.to_string());
            return;
        }
    };
    let tracker_right = match session.create_hand_tracker(openxr::HandEXT::RIGHT) {
        Ok(t) => t,
        Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
            warn!("Handtracking Extension not loaded, Unable to create Handtracker!");
            return;
        }
        Err(err) => {
            warn!("Error while creating Handtracker: {}", err.to_string());
            return;
        }
    };
    let left_bones = spawn_hand_bones(&mut cmds, (DefaultHandBone, LeftHand));
    let right_bones = spawn_hand_bones(&mut cmds, (DefaultHandBone, RightHand));
    cmds.entity(root).push_children(&left_bones);
    cmds.entity(root).push_children(&right_bones);
    cmds.spawn((
        DefaultHandTracker,
        OxrHandTracker(tracker_left),
        XrHandBoneEntities(left_bones),
        LeftHand,
    ));
    cmds.spawn((
        DefaultHandTracker,
        OxrHandTracker(tracker_right),
        XrHandBoneEntities(right_bones),
        RightHand,
    ));
}

#[derive(Component, Clone, Copy)]
struct DefaultHandTracker;
#[derive(Component, Clone, Copy)]
struct DefaultHandBone;

#[allow(clippy::type_complexity)]
fn clean_up_default_hands(
    mut cmds: Commands,
    query: Query<Entity, Or<(With<DefaultHandTracker>, With<DefaultHandBone>)>>,
) {
    for e in &query {
        debug!("removing default hand entity");
        cmds.entity(e).despawn_recursive();
    }
}

#[derive(Deref, DerefMut, Component)]
pub struct OxrHandTracker(pub openxr::HandTracker);

fn locate_hands(
    default_ref_space: Res<XrPrimaryReferenceSpace>,
    frame_state: Res<OxrFrameState>,
    tracker_query: Query<(
        &OxrHandTracker,
        Option<&XrReferenceSpace>,
        &XrHandBoneEntities,
    )>,
    session: Res<OxrSession>,
    mut bone_query: Query<(&HandBone, &mut HandBoneRadius, &mut Transform)>,
    pipelined: Option<Res<Pipelined>>,
) {
    for (tracker, ref_space, hand_entities) in &tracker_query {
        let ref_space = ref_space.map(|v| &v.0).unwrap_or(&default_ref_space.0);
        // relate_hand_joints also provides velocities
        let joints = match session.locate_hand_joints(
            tracker,
            ref_space,
            if pipelined.is_some() {
                openxr::Time::from_nanos(
                    frame_state.predicted_display_time.as_nanos()
                        + frame_state.predicted_display_period.as_nanos(),
                )
            } else {
                frame_state.predicted_display_time
            },
        ) {
            Ok(Some(v)) => v,
            Ok(None) => continue,
            Err(openxr::sys::Result::ERROR_EXTENSION_NOT_PRESENT) => {
                error!("HandTracking Extension not loaded");
                continue;
            }
            Err(err) => {
                warn!("Error while locating hand joints: {}", err.to_string());
                continue;
            }
        };
        let bone_entities = match bone_query.get_many_mut(hand_entities.0) {
            Ok(v) => v,
            Err(err) => {
                warn!("unable to get entities, {}", err);
                continue;
            }
        };
        for (bone, mut bone_radius, mut transform) in bone_entities {
            let joint = joints[*bone as usize];
            **bone_radius = joint.radius;

            if joint
                .location_flags
                .contains(SpaceLocationFlags::POSITION_VALID)
            {
                transform.translation.x = joint.pose.position.x;
                transform.translation.y = joint.pose.position.y;
                transform.translation.z = joint.pose.position.z;
            }

            if joint
                .location_flags
                .contains(SpaceLocationFlags::ORIENTATION_VALID)
            {
                transform.rotation.x = joint.pose.orientation.x;
                transform.rotation.y = joint.pose.orientation.y;
                transform.rotation.z = joint.pose.orientation.z;
                transform.rotation.w = joint.pose.orientation.w;
            }
        }
    }
}
