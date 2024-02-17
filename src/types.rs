use bevy::math::{Quat, Vec3};

#[derive(Debug, Clone)]
pub struct Pose {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub struct Haptic;

#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
    Opaque,
    Additive,
    AlphaBlend,
}
