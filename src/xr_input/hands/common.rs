use bevy::{
    color::{palettes, Srgba}, core::Name, prelude::{
        default, Color, Commands, Component, Deref, DerefMut, Entity, Gizmos, Plugin, PostUpdate,
        Query, Resource, SpatialBundle, Startup, Transform,
    }, transform::components::GlobalTransform
};

use crate::xr_input::{trackers::OpenXRTracker, Hand};

use super::{BoneTrackingStatus, HandBone};

/// add debug renderer for controllers
// #[derive(Default)]
// pub struct OpenXrHandInput;
//
// impl Plugin for OpenXrHandInput {
//     fn build(&self, app: &mut bevy::prelude::App) {
//         app.add_systems(Startup, spawn_hand_entities);
//     }
// }

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
                    Name::new(format!("{:?} {:?}", hand, bone)),
                    SpatialBundle::default(),
                    *bone,
                    OpenXRTracker,
                    *hand,
                    BoneTrackingStatus::Tracked,
                    HandBoneRadius(0.1),
                ))
                .id();
            let hand_res = match hand {
                Hand::Left => &mut hand_resource.left,
                Hand::Right => &mut hand_resource.right,
            };
            match bone {
                HandBone::Palm => hand_res.palm = boneid,
                HandBone::Wrist => hand_res.wrist = boneid,
                HandBone::ThumbMetacarpal => hand_res.thumb.metacarpal = boneid,
                HandBone::ThumbProximal => hand_res.thumb.proximal = boneid,
                HandBone::ThumbDistal => hand_res.thumb.distal = boneid,
                HandBone::ThumbTip => hand_res.thumb.tip = boneid,
                HandBone::IndexMetacarpal => hand_res.index.metacarpal = boneid,
                HandBone::IndexProximal => hand_res.index.proximal = boneid,
                HandBone::IndexIntermediate => hand_res.index.intermediate = boneid,
                HandBone::IndexDistal => hand_res.index.distal = boneid,
                HandBone::IndexTip => hand_res.index.tip = boneid,
                HandBone::MiddleMetacarpal => hand_res.middle.metacarpal = boneid,
                HandBone::MiddleProximal => hand_res.middle.proximal = boneid,
                HandBone::MiddleIntermediate => hand_res.middle.intermediate = boneid,
                HandBone::MiddleDistal => hand_res.middle.distal = boneid,
                HandBone::MiddleTip => hand_res.middle.tip = boneid,
                HandBone::RingMetacarpal => hand_res.ring.metacarpal = boneid,
                HandBone::RingProximal => hand_res.ring.proximal = boneid,
                HandBone::RingIntermediate => hand_res.ring.intermediate = boneid,
                HandBone::RingDistal => hand_res.ring.distal = boneid,
                HandBone::RingTip => hand_res.ring.tip = boneid,
                HandBone::LittleMetacarpal => hand_res.little.metacarpal = boneid,
                HandBone::LittleProximal => hand_res.little.proximal = boneid,
                HandBone::LittleIntermediate => hand_res.little.intermediate = boneid,
                HandBone::LittleDistal => hand_res.little.distal = boneid,
                HandBone::LittleTip => hand_res.little.tip = boneid,
            }
        }
    }
    commands.insert_resource(hand_resource);
}

#[derive(Debug, Component, DerefMut, Deref)]
pub struct HandBoneRadius(pub f32);

pub fn draw_hand_entities(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &HandBone, &HandBoneRadius)>,
) {
    for (transform, hand_bone, hand_bone_radius) in query.iter() {
        let (_, color) = get_bone_gizmo_style(hand_bone);
        let (_, rotation, translation) = transform.to_scale_rotation_translation();
        gizmos.sphere(translation, rotation, hand_bone_radius.0, color);
    }
}

pub(crate) fn get_bone_gizmo_style(hand_bone: &HandBone) -> (f32, Srgba) {
    match hand_bone {
        HandBone::Palm => (0.01, palettes::css::WHITE),
        HandBone::Wrist => (0.01, palettes::css::GRAY),
        HandBone::ThumbMetacarpal => (0.01, palettes::css::RED),
        HandBone::ThumbProximal => (0.008, palettes::css::RED),
        HandBone::ThumbDistal => (0.006, palettes::css::RED),
        HandBone::ThumbTip => (0.004, palettes::css::RED),
        HandBone::IndexMetacarpal => (0.01, palettes::css::ORANGE),
        HandBone::IndexProximal => (0.008, palettes::css::ORANGE),
        HandBone::IndexIntermediate => (0.006, palettes::css::ORANGE),
        HandBone::IndexDistal => (0.004, palettes::css::ORANGE),
        HandBone::IndexTip => (0.002, palettes::css::ORANGE),
        HandBone::MiddleMetacarpal => (0.01, palettes::css::YELLOW),
        HandBone::MiddleProximal => (0.008, palettes::css::YELLOW),
        HandBone::MiddleIntermediate => (0.006, palettes::css::YELLOW),
        HandBone::MiddleDistal => (0.004, palettes::css::YELLOW),
        HandBone::MiddleTip => (0.002, palettes::css::YELLOW),
        HandBone::RingMetacarpal => (0.01, palettes::css::GREEN),
        HandBone::RingProximal => (0.008, palettes::css::GREEN),
        HandBone::RingIntermediate => (0.006, palettes::css::GREEN),
        HandBone::RingDistal => (0.004, palettes::css::GREEN),
        HandBone::RingTip => (0.002, palettes::css::GREEN),
        HandBone::LittleMetacarpal => (0.01, palettes::css::BLUE),
        HandBone::LittleProximal => (0.008, palettes::css::BLUE),
        HandBone::LittleIntermediate => (0.006, palettes::css::BLUE),
        HandBone::LittleDistal => (0.004, palettes::css::BLUE),
        HandBone::LittleTip => (0.002, palettes::css::BLUE),
    }
}
