use bevy::prelude::*;
use openxr::{HandTracker, Result, SpaceLocationFlags};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
    xr_input::{
        hand::HandBoneRadius, hands::HandBone, trackers::OpenXRTrackingRoot, Hand, QuatConv,
        Vec3Conv,
    },
};

use super::BoneTrackingStatus;

#[derive(Resource, PartialEq)]
pub enum DisableHandTracking {
    OnlyLeft,
    OnlyRight,
    Both,
}
pub struct HandTrackingPlugin;

#[derive(Resource)]
pub struct HandTrackingData {
    left_hand: HandTracker,
    right_hand: HandTracker,
}

impl HandTrackingData {
    pub fn new(session: &XrSession) -> Result<HandTrackingData> {
        let left = session.create_hand_tracker(openxr::HandEXT::LEFT)?;
        let right = session.create_hand_tracker(openxr::HandEXT::RIGHT)?;
        Ok(HandTrackingData {
            left_hand: left,
            right_hand: right,
        })
    }
    pub fn get_ref<'a>(
        &'a self,
        input: &'a XrInput,
        frame_state: &'a XrFrameState,
    ) -> HandTrackingRef<'a> {
        HandTrackingRef {
            tracking: self,
            input,
            frame_state,
        }
    }
}

pub struct HandTrackingRef<'a> {
    tracking: &'a HandTrackingData,
    input: &'a XrInput,
    frame_state: &'a XrFrameState,
}
#[derive(Debug)]
pub struct HandJoint {
    position: Vec3,
    position_valid: bool,
    position_tracked: bool,
    orientaion: Quat,
    orientaion_valid: bool,
    orientaion_tracked: bool,
    radius: f32,
}

pub struct HandJoints {
    inner: [HandJoint; 26],
}

impl HandJoints {
    pub fn get_joint(&self, bone: HandBone) -> &HandJoint {
        &self.inner[bone.get_index_from_bone()]
    }
}

impl<'a> HandTrackingRef<'a> {
    pub fn get_poses(&self, side: Hand) -> Option<HandJoints> {
        self.input
            .stage
            .locate_hand_joints(
                match side {
                    Hand::Left => &self.tracking.left_hand,
                    Hand::Right => &self.tracking.right_hand,
                },
                self.frame_state.lock().unwrap().predicted_display_time,
            )
            .unwrap()
            .map(|joints| {
                joints
                    .into_iter()
                    .map(|joint| HandJoint {
                        position: joint.pose.position.to_vec3(),
                        orientaion: joint.pose.orientation.to_quat(),
                        position_valid: joint
                            .location_flags
                            .contains(SpaceLocationFlags::POSITION_VALID),
                        position_tracked: joint
                            .location_flags
                            .contains(SpaceLocationFlags::POSITION_TRACKED),
                        orientaion_valid: joint
                            .location_flags
                            .contains(SpaceLocationFlags::ORIENTATION_VALID),
                        orientaion_tracked: joint
                            .location_flags
                            .contains(SpaceLocationFlags::ORIENTATION_TRACKED),
                        radius: joint.radius,
                    })
                    .collect::<Vec<HandJoint>>()
                    .try_into()
                    .unwrap()
            })
            .map(|joints| HandJoints { inner: joints })
    }
}

impl Plugin for HandTrackingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_hand_bones).run_if(|dh: Option<Res<DisableHandTracking>>| {
                !dh.is_some_and(|v| *v == DisableHandTracking::Both)
            }),
        );
    }
}

pub fn update_hand_bones(
    hand_tracking: Res<HandTrackingData>,
    xr_input: Res<XrInput>,
    xr_frame_state: Res<XrFrameState>,
    root_query: Query<(&Transform, With<OpenXRTrackingRoot>, Without<HandBone>)>,
    mut bones: Query<(
        &mut Transform,
        &Hand,
        &HandBone,
        &mut HandBoneRadius,
        &mut BoneTrackingStatus,
    )>,
) {
    let hand_ref = hand_tracking.get_ref(&xr_input, &xr_frame_state);
    let (root_transform, _, _) = root_query.get_single().unwrap();
    let left_hand_data = hand_ref.get_poses(Hand::Left);
    let right_hand_data = hand_ref.get_poses(Hand::Right);
    bones
        .par_iter_mut()
        .for_each(|(mut transform, hand, bone, mut radius, mut status)| {
            let bone_data = match (hand, &left_hand_data, &right_hand_data) {
                (Hand::Left, Some(data), _) => data.get_joint(*bone),
                (Hand::Right, _, Some(data)) => data.get_joint(*bone),
                _ => {
                    *status = BoneTrackingStatus::Emulated;
                    return;
                }
            };
            if *status == BoneTrackingStatus::Emulated {
                *status = BoneTrackingStatus::Tracked;
            }
            radius.0 = bone_data.radius;
            *transform = transform
                .with_translation(root_transform.transform_point(bone_data.position))
                .with_rotation(root_transform.rotation * bone_data.orientaion)
        });
}
