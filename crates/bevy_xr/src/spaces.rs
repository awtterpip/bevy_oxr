use bevy_camera::visibility::Visibility;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::{component::Component, message::Message, resource::Resource, schedule::SystemSet};
use bevy_math::Vec3;
use bevy_render::{extract_component::ExtractComponent, extract_resource::ExtractResource};
use bevy_transform::components::Transform;
#[cfg(feature="reflect")]
use bevy_reflect::Reflect;

use crate::session::XrTracker;

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct XrSpaceSyncSet;

/// Any Spaces will be invalid after the owning session exits
#[repr(transparent)]
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Debug, ExtractComponent)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[require(XrSpaceLocationFlags, Transform, Visibility, XrTracker)]
pub struct XrSpace(u64);

#[derive(Component, Clone, Copy, Debug, ExtractComponent, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[require(XrSpaceVelocityFlags)]
pub struct XrVelocity {
    /// Velocity of a space relative to it's reference space
    pub linear: Vec3,
    /// Angular Velocity of a space relative to it's reference space
    /// the direction of the vector is parrelel to the axis of rotation,
    /// the magnitude is the relative angular speed in radians per second
    /// the vector follows the right-hand rule for torque/rotation
    pub angular: Vec3,
}
impl XrVelocity {
    pub const fn new() -> XrVelocity {
        XrVelocity {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        }
    }
}

#[derive(Message, Clone, Copy, Deref, DerefMut)]
pub struct XrDestroySpace(pub XrSpace);

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Component, Deref, DerefMut, ExtractComponent)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct XrReferenceSpace(pub XrSpace);

#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Resource, Deref, DerefMut, ExtractResource)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct XrPrimaryReferenceSpace(pub XrReferenceSpace);

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug, Component, ExtractComponent, Default)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct XrSpaceLocationFlags {
    pub position_tracked: bool,
    pub rotation_tracked: bool,
}

#[derive(
    Clone, Copy, Hash, PartialEq, Eq, Debug, Component, ExtractComponent, Default,
)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
pub struct XrSpaceVelocityFlags {
    pub linear_valid: bool,
    pub angular_valid: bool,
}

impl XrSpace {
    /// # Safety
    /// only call with known valid handles
    pub unsafe fn from_raw(handle: u64) -> Self {
        Self(handle)
    }
    pub fn as_raw(&self) -> u64 {
        self.0
    }
}
