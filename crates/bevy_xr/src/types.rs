use bevy::{
    math::{Quat, Vec3},
    reflect::Reflect,
    transform::components::Transform,
};

#[derive(Clone, Copy, PartialEq, Reflect, Debug)]
pub struct XrPose {
    pub translation: Vec3,
    pub rotation: Quat,
}

impl Default for XrPose {
    fn default() -> Self {
        Self::IDENTITY
    }
}
impl XrPose {
    pub const IDENTITY: XrPose = XrPose {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
    };
    pub const fn to_transform(self) -> Transform {
        Transform::from_translation(self.translation).with_rotation(self.rotation)
    }
}
