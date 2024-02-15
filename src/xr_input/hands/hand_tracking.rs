use bevy::prelude::*;
use openxr::{HandTracker, Result, SpaceLocationFlags};

use super::common::HandBoneRadius;
use crate::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
    xr_init::xr_only,
    xr_input::{hands::HandBone, Hand, QuatConv, Vec3Conv},
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
    pub position: Vec3,
    pub position_valid: bool,
    pub position_tracked: bool,
    pub orientation: Quat,
    pub orientation_valid: bool,
    pub orientation_tracked: bool,
    pub radius: f32,
}

#[derive(Debug)]
pub struct HandJoints {
    inner: [HandJoint; 26],
}
impl HandJoints {
    pub fn inner(&self) -> &[HandJoint; 26] {
        &self.inner
    }
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
                self.frame_state.predicted_display_time,
            )
            .unwrap()
            .map(|joints| {
                joints
                    .into_iter()
                    .map(|joint| HandJoint {
                        position: joint.pose.position.to_vec3(),
                        orientation: joint.pose.orientation.to_quat(),
                        position_valid: joint
                            .location_flags
                            .contains(SpaceLocationFlags::POSITION_VALID),
                        position_tracked: joint
                            .location_flags
                            .contains(SpaceLocationFlags::POSITION_TRACKED),
                        orientation_valid: joint
                            .location_flags
                            .contains(SpaceLocationFlags::ORIENTATION_VALID),
                        orientation_tracked: joint
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
            PreUpdate,
            (
                update_hand_bones
                    .run_if(|dh: Option<Res<DisableHandTracking>>| {
                        !dh.is_some_and(|v| *v == DisableHandTracking::Both)
                    })
                    .run_if(xr_only()),
                update_tracking_state_on_disable,
            ),
        );
    }
}

fn update_tracking_state_on_disable(
    mut is_off: Local<bool>,
    disabled_tracking: Option<Res<DisableHandTracking>>,
    mut tracking_states: Query<&mut BoneTrackingStatus>,
) {
    if !*is_off
        && disabled_tracking
            .as_ref()
            .is_some_and(|t| **t == DisableHandTracking::Both)
    {
        tracking_states
            .par_iter_mut()
            .for_each(|mut state| *state = BoneTrackingStatus::Emulated);
    }
    *is_off = disabled_tracking
        .as_ref()
        .is_some_and(|t| **t == DisableHandTracking::Both);
}

pub fn update_hand_bones(
    disabled_tracking: Option<Res<DisableHandTracking>>,
    hand_tracking: Option<Res<HandTrackingData>>,
    xr_input: Res<XrInput>,
    xr_frame_state: Res<XrFrameState>,
    mut bones: Query<(
        &mut Transform,
        &Hand,
        &HandBone,
        &mut HandBoneRadius,
        &mut BoneTrackingStatus,
    )>,
) {
    let hand_ref = match hand_tracking.as_ref() {
        Some(h) => h.get_ref(&xr_input, &xr_frame_state),
        None => {
            warn!("No Handtracking data!");
            return;
        }
    };
    let left_hand_data = hand_ref.get_poses(Hand::Left);
    let right_hand_data = hand_ref.get_poses(Hand::Right);
    if left_hand_data.is_none() || right_hand_data.is_none() {
        error!("something is very wrong for hand_tracking!! doesn't have data for both hands!");
    }

    info!("hand_tracking");
    bones
        .par_iter_mut()
        .for_each(|(mut transform, hand, bone, mut radius, mut status)| {
            info!("hand_tracking bone before filter");
            match (&hand, disabled_tracking.as_ref().map(|d| d.as_ref())) {
                (Hand::Left, Some(DisableHandTracking::OnlyLeft)) => {
                    *status = BoneTrackingStatus::Emulated;
                    return;
                }
                (Hand::Right, Some(DisableHandTracking::OnlyRight)) => {
                    *status = BoneTrackingStatus::Emulated;
                    return;
                }
                _ => {}
            }
            info!("hand_tracking bone mid filter");
            let bone_data = match (hand, &left_hand_data, &right_hand_data) {
                (Hand::Left, Some(data), _) => data.get_joint(*bone),
                (Hand::Right, _, Some(data)) => data.get_joint(*bone),
                (hand, left_data, right_data) => {
                    info!("{:?},{:?},{:?}", hand, left_data, right_data);
                    *status = BoneTrackingStatus::Emulated;
                    return;
                }
            };
            info!("hand_tracking bone after filter");
            if *status == BoneTrackingStatus::Emulated {
                *status = BoneTrackingStatus::Tracked;
            }
            radius.0 = bone_data.radius;
            transform.translation = bone_data.position;
            transform.rotation = bone_data.orientation;
        });
}
