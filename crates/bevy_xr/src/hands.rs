use bevy::{
    ecs::component::Component,
    math::bool,
    prelude::{Deref, DerefMut},
};

#[derive(Clone, Copy, Component, Debug)]
pub struct LeftHand;

#[derive(Clone, Copy, Component, Debug)]
pub struct RightHand;

#[repr(transparent)]
#[derive(Clone, Copy, Component, Debug, DerefMut, Deref)]
pub struct HandBoneRadius(pub f32);

#[repr(u8)]
#[derive(Clone, Copy, Component, Debug)]
pub enum HandBone {
    Palm = 0,
    Wrist = 1,
    ThumbMetacarpal = 2,
    ThumbProximal = 3,
    ThumbDistal = 4,
    ThumbTip = 5,
    IndexMetacarpal = 6,
    IndexProximal = 7,
    IndexIntermediate = 8,
    IndexDistal = 9,
    IndexTip = 10,
    MiddleMetacarpal = 11,
    MiddleProximal = 12,
    MiddleIntermediate = 13,
    MiddleDistal = 14,
    MiddleTip = 15,
    RingMetacarpal = 16,
    RingProximal = 17,
    RingIntermediate = 18,
    RingDistal = 19,
    RingTip = 20,
    LittleMetacarpal = 21,
    LittleProximal = 22,
    LittleIntermediate = 23,
    LittleDistal = 24,
    LittleTip = 25,
}

impl HandBone {
    pub const fn is_metacarpal(&self) -> bool {
        matches!(
            self,
            HandBone::ThumbMetacarpal
                | HandBone::IndexMetacarpal
                | HandBone::MiddleMetacarpal
                | HandBone::RingMetacarpal
                | HandBone::LittleMetacarpal
        )
    }
    pub const fn is_thumb(&self) -> bool {
        matches!(
            self,
            HandBone::ThumbMetacarpal
                | HandBone::ThumbProximal
                | HandBone::ThumbDistal
                | HandBone::ThumbTip
        )
    }
    pub const fn is_index(&self) -> bool {
        matches!(
            self,
            HandBone::IndexMetacarpal
                | HandBone::IndexProximal
                | HandBone::IndexIntermediate
                | HandBone::IndexDistal
                | HandBone::IndexTip
        )
    }
    pub const fn is_middle(&self) -> bool {
        matches!(
            self,
            HandBone::MiddleMetacarpal
                | HandBone::MiddleProximal
                | HandBone::MiddleIntermediate
                | HandBone::MiddleDistal
                | HandBone::MiddleTip
        )
    }
    pub const fn is_ring(&self) -> bool {
        matches!(
            self,
            HandBone::RingMetacarpal
                | HandBone::RingProximal
                | HandBone::RingIntermediate
                | HandBone::RingDistal
                | HandBone::RingTip
        )
    }
    pub const fn is_little(&self) -> bool {
        matches!(
            self,
            HandBone::LittleMetacarpal
                | HandBone::LittleProximal
                | HandBone::LittleIntermediate
                | HandBone::LittleDistal
                | HandBone::LittleTip
        )
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
}
