use bevy::math::{Quat, Vec3};

pub struct Pose {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub struct Haptic;
