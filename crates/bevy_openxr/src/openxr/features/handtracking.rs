use bevy::prelude::*;
use bevy_xr::{
    hands::{HandBone, HandBoneRadius},
    session::{session_running, status_changed_to, XrStatus},
};
use openxr::SpaceLocationFlags;

use crate::{
    init::OxrTrackingRoot, reference_space::{OxrPrimaryReferenceSpace, OxrReferenceSpace}, resources::{OxrSession, OxrTime}
};

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
            app.add_systems(
                PreUpdate,
                clean_up_default_hands.run_if(status_changed_to(XrStatus::Exiting)),
            );
            app.add_systems(
                PostUpdate,
                spawn_default_hands.run_if(status_changed_to(XrStatus::Ready)),
            );
        }
    }
}

fn spawn_default_hands(
    mut cmds: Commands,
    session: Res<OxrSession>,
    root: Query<Entity, With<OxrTrackingRoot>>,
) {
    info!("spawning hands");
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
    let mut left_bones = [Entity::PLACEHOLDER; 26];
    let mut right_bones = [Entity::PLACEHOLDER; 26];
    for bone in HandBone::get_all_bones() {
        let bone_left = cmds
            .spawn((SpatialBundle::default(), bone, HandBoneRadius(0.0)))
            .id();
        let bone_right = cmds
            .spawn((SpatialBundle::default(), bone, HandBoneRadius(0.0)))
            .id();
        cmds.entity(root).push_children(&[bone_right]);
        left_bones[bone as usize] = bone_left;
        right_bones[bone as usize] = bone_right;
    }
    cmds.spawn((
        DefaultHandTracker,
        OxrHandTracker(tracker_left),
        OxrHandBoneEntities(left_bones),
    ));
    cmds.spawn((
        DefaultHandTracker,
        OxrHandTracker(tracker_right),
        OxrHandBoneEntities(right_bones),
    ));
}

#[derive(Component)]
struct DefaultHandTracker;
#[derive(Component)]
struct DefaultHandBones;

#[allow(clippy::type_complexity)]
fn clean_up_default_hands(
    mut cmds: Commands,
    query: Query<Entity, Or<(With<DefaultHandTracker>, With<DefaultHandBones>)>>,
) {
    for e in &query {
        cmds.entity(e).despawn();
    }
}

#[derive(Deref, DerefMut, Component, Clone, Copy)]
pub struct OxrHandBoneEntities(pub [Entity; 26]);

#[derive(Deref, DerefMut, Component)]
pub struct OxrHandTracker(pub openxr::HandTracker);

fn locate_hands(
    default_ref_space: Res<OxrPrimaryReferenceSpace>,
    time: Res<OxrTime>,
    tracker_query: Query<(
        &OxrHandTracker,
        Option<&OxrReferenceSpace>,
        &OxrHandBoneEntities,
    )>,
    mut bone_query: Query<(&HandBone, &mut HandBoneRadius, &mut Transform)>,
) {
    info!("updating hands 0 ");
    for (tracker, ref_space, hand_entities) in &tracker_query {
        info!("updating hands 1 ");
        let ref_space = ref_space.map(|v| &v.0).unwrap_or(&default_ref_space.0);
        // relate_hand_joints also provides velocities
        let joints = match ref_space.locate_hand_joints(tracker, **time) {
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
            info!("updating hands");
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
