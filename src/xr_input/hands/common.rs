use bevy::prelude::{
    default, Color, Commands, Component, Deref, DerefMut, Entity, Gizmos, Plugin, PostUpdate,
    Query, Resource, SpatialBundle, Startup, Transform,
};

use crate::xr_input::{Hand, trackers::OpenXRTracker};

use super::{HandBone, BoneTrackingStatus};

/// add debug renderer for controllers
#[derive(Default)]
pub struct OpenXrHandInput;

impl Plugin for OpenXrHandInput {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_hand_entities);
    }
}

/// add debug renderer for controllers
#[derive(Default)]
pub struct HandInputDebugRenderer;

impl Plugin for HandInputDebugRenderer {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(PostUpdate, draw_hand_entities);
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct HandsResource {
    pub left: HandResource,
    pub right: HandResource,
}
#[derive(Clone, Copy)]
pub struct HandResource {
    pub palm: Entity,
    pub wrist: Entity,
    pub thumb: ThumbResource,
    pub index: IndexResource,
    pub middle: MiddleResource,
    pub ring: RingResource,
    pub little: LittleResource,
}

impl Default for HandResource {
    fn default() -> Self {
        Self {
            palm: Entity::PLACEHOLDER,
            wrist: Entity::PLACEHOLDER,
            thumb: Default::default(),
            index: Default::default(),
            middle: Default::default(),
            ring: Default::default(),
            little: Default::default(),
        }
    }
}
#[derive(Clone, Copy)]
pub struct ThumbResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for ThumbResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct IndexResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for IndexResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct MiddleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for MiddleResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct RingResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for RingResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}
#[derive(Clone, Copy)]
pub struct LittleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for LittleResource {
    fn default() -> Self {
        Self {
            metacarpal: Entity::PLACEHOLDER,
            proximal: Entity::PLACEHOLDER,
            intermediate: Entity::PLACEHOLDER,
            distal: Entity::PLACEHOLDER,
            tip: Entity::PLACEHOLDER,
        }
    }
}

pub fn spawn_hand_entities(mut commands: Commands) {
    let hands = [Hand::Left, Hand::Right];
    let bones = HandBone::get_all_bones();
    //hand resource
    let mut hand_resource = HandsResource { ..default() };
    for hand in hands.iter() {
        for bone in bones.iter() {
            let boneid = commands
                .spawn((
                    SpatialBundle::default(),
                    bone.clone(),
                    OpenXRTracker,
                    hand.clone(),
                    BoneTrackingStatus::Emulated,
                    HandBoneRadius(0.1),
                ))
                .id();
            match hand {
                Hand::Left => match bone {
                    HandBone::Palm => hand_resource.left.palm = boneid,
                    HandBone::Wrist => hand_resource.left.wrist = boneid,
                    HandBone::ThumbMetacarpal => hand_resource.left.thumb.metacarpal = boneid,
                    HandBone::ThumbProximal => hand_resource.left.thumb.proximal = boneid,
                    HandBone::ThumbDistal => hand_resource.left.thumb.distal = boneid,
                    HandBone::ThumbTip => hand_resource.left.thumb.tip = boneid,
                    HandBone::IndexMetacarpal => hand_resource.left.index.metacarpal = boneid,
                    HandBone::IndexProximal => hand_resource.left.index.proximal = boneid,
                    HandBone::IndexIntermediate => hand_resource.left.index.intermediate = boneid,
                    HandBone::IndexDistal => hand_resource.left.index.distal = boneid,
                    HandBone::IndexTip => hand_resource.left.index.tip = boneid,
                    HandBone::MiddleMetacarpal => hand_resource.left.middle.metacarpal = boneid,
                    HandBone::MiddleProximal => hand_resource.left.middle.proximal = boneid,
                    HandBone::MiddleIntermediate => hand_resource.left.middle.intermediate = boneid,
                    HandBone::MiddleDistal => hand_resource.left.middle.distal = boneid,
                    HandBone::MiddleTip => hand_resource.left.middle.tip = boneid,
                    HandBone::RingMetacarpal => hand_resource.left.ring.metacarpal = boneid,
                    HandBone::RingProximal => hand_resource.left.ring.proximal = boneid,
                    HandBone::RingIntermediate => hand_resource.left.ring.intermediate = boneid,
                    HandBone::RingDistal => hand_resource.left.ring.distal = boneid,
                    HandBone::RingTip => hand_resource.left.ring.tip = boneid,
                    HandBone::LittleMetacarpal => hand_resource.left.little.metacarpal = boneid,
                    HandBone::LittleProximal => hand_resource.left.little.proximal = boneid,
                    HandBone::LittleIntermediate => hand_resource.left.little.intermediate = boneid,
                    HandBone::LittleDistal => hand_resource.left.little.distal = boneid,
                    HandBone::LittleTip => hand_resource.left.little.tip = boneid,
                },
                Hand::Right => match bone {
                    HandBone::Palm => hand_resource.right.palm = boneid,
                    HandBone::Wrist => hand_resource.right.wrist = boneid,
                    HandBone::ThumbMetacarpal => hand_resource.right.thumb.metacarpal = boneid,
                    HandBone::ThumbProximal => hand_resource.right.thumb.proximal = boneid,
                    HandBone::ThumbDistal => hand_resource.right.thumb.distal = boneid,
                    HandBone::ThumbTip => hand_resource.right.thumb.tip = boneid,
                    HandBone::IndexMetacarpal => hand_resource.right.index.metacarpal = boneid,
                    HandBone::IndexProximal => hand_resource.right.index.proximal = boneid,
                    HandBone::IndexIntermediate => hand_resource.right.index.intermediate = boneid,
                    HandBone::IndexDistal => hand_resource.right.index.distal = boneid,
                    HandBone::IndexTip => hand_resource.right.index.tip = boneid,
                    HandBone::MiddleMetacarpal => hand_resource.right.middle.metacarpal = boneid,
                    HandBone::MiddleProximal => hand_resource.right.middle.proximal = boneid,
                    HandBone::MiddleIntermediate => {
                        hand_resource.right.middle.intermediate = boneid
                    }
                    HandBone::MiddleDistal => hand_resource.right.middle.distal = boneid,
                    HandBone::MiddleTip => hand_resource.right.middle.tip = boneid,
                    HandBone::RingMetacarpal => hand_resource.right.ring.metacarpal = boneid,
                    HandBone::RingProximal => hand_resource.right.ring.proximal = boneid,
                    HandBone::RingIntermediate => hand_resource.right.ring.intermediate = boneid,
                    HandBone::RingDistal => hand_resource.right.ring.distal = boneid,
                    HandBone::RingTip => hand_resource.right.ring.tip = boneid,
                    HandBone::LittleMetacarpal => hand_resource.right.little.metacarpal = boneid,
                    HandBone::LittleProximal => hand_resource.right.little.proximal = boneid,
                    HandBone::LittleIntermediate => {
                        hand_resource.right.little.intermediate = boneid
                    }
                    HandBone::LittleDistal => hand_resource.right.little.distal = boneid,
                    HandBone::LittleTip => hand_resource.right.little.tip = boneid,
                },
            }
        }
    }
    commands.insert_resource(hand_resource);
}

#[derive(Debug, Component, DerefMut, Deref)]
pub struct HandBoneRadius(pub f32);

pub fn draw_hand_entities(
    mut gizmos: Gizmos,
    query: Query<(&Transform, &HandBone, &HandBoneRadius)>,
) {
    for (transform, hand_bone, hand_bone_radius) in query.iter() {
        let (_, color) = get_bone_gizmo_style(hand_bone);
        gizmos.sphere(
            transform.translation,
            transform.rotation,
            hand_bone_radius.0,
            color,
        );
    }
}

pub(crate) fn get_bone_gizmo_style(hand_bone: &HandBone) -> (f32, Color) {
    match hand_bone {
        HandBone::Palm => (0.01, Color::WHITE),
        HandBone::Wrist => (0.01, Color::GRAY),
        HandBone::ThumbMetacarpal => (0.01, Color::RED),
        HandBone::ThumbProximal => (0.008, Color::RED),
        HandBone::ThumbDistal => (0.006, Color::RED),
        HandBone::ThumbTip => (0.004, Color::RED),
        HandBone::IndexMetacarpal => (0.01, Color::ORANGE),
        HandBone::IndexProximal => (0.008, Color::ORANGE),
        HandBone::IndexIntermediate => (0.006, Color::ORANGE),
        HandBone::IndexDistal => (0.004, Color::ORANGE),
        HandBone::IndexTip => (0.002, Color::ORANGE),
        HandBone::MiddleMetacarpal => (0.01, Color::YELLOW),
        HandBone::MiddleProximal => (0.008, Color::YELLOW),
        HandBone::MiddleIntermediate => (0.006, Color::YELLOW),
        HandBone::MiddleDistal => (0.004, Color::YELLOW),
        HandBone::MiddleTip => (0.002, Color::YELLOW),
        HandBone::RingMetacarpal => (0.01, Color::GREEN),
        HandBone::RingProximal => (0.008, Color::GREEN),
        HandBone::RingIntermediate => (0.006, Color::GREEN),
        HandBone::RingDistal => (0.004, Color::GREEN),
        HandBone::RingTip => (0.002, Color::GREEN),
        HandBone::LittleMetacarpal => (0.01, Color::BLUE),
        HandBone::LittleProximal => (0.008, Color::BLUE),
        HandBone::LittleIntermediate => (0.006, Color::BLUE),
        HandBone::LittleDistal => (0.004, Color::BLUE),
        HandBone::LittleTip => (0.002, Color::BLUE),
    }
}
