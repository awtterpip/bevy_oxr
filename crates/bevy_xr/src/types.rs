use bevy::math::{Quat, Vec3};

pub struct XrPose {
    pub position: Vec3,
    pub rotation: Quat,
}

impl XrPose {
    pub const IDENTITY: XrPose = XrPose {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };
}
