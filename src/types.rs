use bevy::ecs::component::Component;
use bevy::math::{Quat, Vec3};
use bevy::render::camera::ManualTextureViewHandle;

#[derive(Debug, Clone)]
pub struct Pose {
    pub translation: Vec3,
    pub rotation: Quat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub struct XrView {
    pub view_handle: ManualTextureViewHandle,
    pub view_index: usize,
}

pub struct Haptic;

#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Opaque,
    Additive,
    AlphaBlend,
}
