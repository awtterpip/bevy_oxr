use bevy::{app::PluginGroupBuilder, prelude::*};
use openxr::FormFactor;

use crate::{
    resources::{XrInstance, XrSession},
    xr_init::XrPreSetup,
};

use self::{
    emulated::HandEmulationPlugin,
    hand_tracking::{DisableHandTracking, HandTrackingData, HandTrackingPlugin},
};

pub mod common;
pub mod emulated;
pub mod hand_tracking;

pub struct XrHandPlugins;

impl Plugin for XrHandPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(HandTrackingPlugin)
            .add_plugins(HandPlugin)
            .add_plugins(HandEmulationPlugin);
    }
}

pub struct HandPlugin;

impl Plugin for HandPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(XrPreSetup, check_for_handtracking);
    }
}

fn check_for_handtracking(
    mut commands: Commands,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
) {
    let hands = instance.exts().ext_hand_tracking.is_some()
        && instance
            .supports_hand_tracking(instance.system(FormFactor::HEAD_MOUNTED_DISPLAY).unwrap())
            .is_ok_and(|v| v);
    if hands {
        commands.insert_resource(HandTrackingData::new(&session).unwrap());
    } else {
        commands.insert_resource(DisableHandTracking::Both);
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub enum BoneTrackingStatus {
    Emulated,
    Tracked,
}

#[derive(Component, Debug, Clone, Copy)]
pub enum HandBone {
    Palm,
    Wrist,
    ThumbMetacarpal,
    ThumbProximal,
    ThumbDistal,
    ThumbTip,
    IndexMetacarpal,
    IndexProximal,
    IndexIntermediate,
    IndexDistal,
    IndexTip,
    MiddleMetacarpal,
    MiddleProximal,
    MiddleIntermediate,
    MiddleDistal,
    MiddleTip,
    RingMetacarpal,
    RingProximal,
    RingIntermediate,
    RingDistal,
    RingTip,
    LittleMetacarpal,
    LittleProximal,
    LittleIntermediate,
    LittleDistal,
    LittleTip,
}
impl HandBone {
    pub fn is_finger(&self) -> bool {
        match &self {
            HandBone::Wrist => false,
            HandBone::Palm => false,
            _ => true,
        }
    }
    pub fn is_metacarpal(&self) -> bool {
        match &self {
            HandBone::ThumbMetacarpal => true,
            HandBone::IndexMetacarpal => true,
            HandBone::MiddleMetacarpal => true,
            HandBone::RingMetacarpal => true,
            HandBone::LittleTip => true,
            _ => false,
        }
    }
    pub const fn get_all_bones() -> [HandBone; 26] {
        [
            HandBone::Palm,
            HandBone::Wrist,
            HandBone::ThumbMetacarpal,
            HandBone::ThumbProximal,
            HandBone::ThumbDistal,
            HandBone::ThumbTip,
            HandBone::IndexMetacarpal,
            HandBone::IndexProximal,
            HandBone::IndexIntermediate,
            HandBone::IndexDistal,
            HandBone::IndexTip,
            HandBone::MiddleMetacarpal,
            HandBone::MiddleProximal,
            HandBone::MiddleIntermediate,
            HandBone::MiddleDistal,
            HandBone::MiddleTip,
            HandBone::RingMetacarpal,
            HandBone::RingProximal,
            HandBone::RingIntermediate,
            HandBone::RingDistal,
            HandBone::RingTip,
            HandBone::LittleMetacarpal,
            HandBone::LittleProximal,
            HandBone::LittleIntermediate,
            HandBone::LittleDistal,
            HandBone::LittleTip,
        ]
    }
    pub fn get_index_from_bone(&self) -> usize {
        match &self {
            HandBone::Palm => 0,
            HandBone::Wrist => 1,
            HandBone::ThumbMetacarpal => 2,
            HandBone::ThumbProximal => 3,
            HandBone::ThumbDistal => 4,
            HandBone::ThumbTip => 5,
            HandBone::IndexMetacarpal => 6,
            HandBone::IndexProximal => 7,
            HandBone::IndexIntermediate => 8,
            HandBone::IndexDistal => 9,
            HandBone::IndexTip => 10,
            HandBone::MiddleMetacarpal => 11,
            HandBone::MiddleProximal => 12,
            HandBone::MiddleIntermediate => 13,
            HandBone::MiddleDistal => 14,
            HandBone::MiddleTip => 15,
            HandBone::RingMetacarpal => 16,
            HandBone::RingProximal => 17,
            HandBone::RingIntermediate => 18,
            HandBone::RingDistal => 19,
            HandBone::RingTip => 20,
            HandBone::LittleMetacarpal => 21,
            HandBone::LittleProximal => 22,
            HandBone::LittleIntermediate => 23,
            HandBone::LittleDistal => 24,
            HandBone::LittleTip => 25,
        }
    }
}
