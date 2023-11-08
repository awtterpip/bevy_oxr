use bevy::prelude::*;
use openxr::{HandJointLocationEXT, HandTracker, Result};

use crate::{
    input::XrInput,
    resources::{XrFrameState, XrSession},
};

#[derive(Resource)]
pub struct HandTrackingTracker {
    left_hand: HandTracker,
    right_hand: HandTracker,
}

impl HandTrackingTracker {
    pub fn new(session: &XrSession) -> Result<HandTrackingTracker> {
        let left = session.create_hand_tracker(openxr::HandEXT::LEFT)?;
        let right = session.create_hand_tracker(openxr::HandEXT::RIGHT)?;
        Ok(HandTrackingTracker {
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
    tracking: &'a HandTrackingTracker,
    input: &'a XrInput,
    frame_state: &'a XrFrameState,
}

// pub type HandJoints = [(HandJointLocationEXT, HandBone); 26];

impl<'a> HandTrackingRef<'a> {
    pub fn get_left_poses(&self) -> Option<[HandJointLocationEXT; 26]> {
        self.input
            .stage
            .locate_hand_joints(
                &self.tracking.left_hand,
                self.frame_state.lock().unwrap().predicted_display_time,
            )
            .unwrap()
        // .map(|joints| {
        //     joints
        //         .into_iter()
        //         .zip(HandBone::get_all_bones().into_iter())
        //         .collect::<Vec<(HandJointLocationEXT, HandBone)>>()
        //         .try_into()
        //         .unwrap()
        // })
    }
    pub fn get_right_poses(&self) -> Option<[HandJointLocationEXT; 26]> {
        self.input
            .stage
            .locate_hand_joints(
                &self.tracking.right_hand,
                self.frame_state.lock().unwrap().predicted_display_time,
            )
            .unwrap()
        // .map(|joints| {
        //     joints
        //         .into_iter()
        //         .zip(HandBone::get_all_bones().into_iter())
        //         .collect::<Vec<(HandJointLocationEXT, HandBone)>>()
        //         .try_into()
        //         .unwrap()
        // })
    }
}
