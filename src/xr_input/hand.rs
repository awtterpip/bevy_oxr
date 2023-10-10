use bevy::prelude::{Entity, Resource};

#[derive(Resource, Default)]
pub struct HandsResource {
    pub left: HandResource,
    pub right: HandResource,
}

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
        Self { palm: Entity::PLACEHOLDER, wrist: Entity::PLACEHOLDER, thumb: Default::default(), index: Default::default(), middle: Default::default(), ring: Default::default(), little: Default::default() }
    }
}

pub struct ThumbResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for ThumbResource {
    fn default() -> Self {
        Self { metacarpal: Entity::PLACEHOLDER, proximal: Entity::PLACEHOLDER, distal: Entity::PLACEHOLDER, tip: Entity::PLACEHOLDER }
    }
}
pub struct IndexResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}

impl Default for IndexResource {
    fn default() -> Self {
        Self { metacarpal: Entity::PLACEHOLDER, proximal: Entity::PLACEHOLDER, intermediate: Entity::PLACEHOLDER, distal: Entity::PLACEHOLDER, tip: Entity::PLACEHOLDER }
    }
}
pub struct MiddleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for MiddleResource {
    fn default() -> Self {
        Self { metacarpal: Entity::PLACEHOLDER, proximal: Entity::PLACEHOLDER, intermediate: Entity::PLACEHOLDER, distal: Entity::PLACEHOLDER, tip: Entity::PLACEHOLDER }
    }
}
pub struct RingResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for RingResource {
    fn default() -> Self {
        Self { metacarpal: Entity::PLACEHOLDER, proximal: Entity::PLACEHOLDER, intermediate: Entity::PLACEHOLDER, distal: Entity::PLACEHOLDER, tip: Entity::PLACEHOLDER }
    }
}
pub struct LittleResource {
    pub metacarpal: Entity,
    pub proximal: Entity,
    pub intermediate: Entity,
    pub distal: Entity,
    pub tip: Entity,
}
impl Default for LittleResource {
    fn default() -> Self {
        Self { metacarpal: Entity::PLACEHOLDER, proximal: Entity::PLACEHOLDER, intermediate: Entity::PLACEHOLDER, distal: Entity::PLACEHOLDER, tip: Entity::PLACEHOLDER }
    }
}
