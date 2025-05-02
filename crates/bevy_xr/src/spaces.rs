use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, extract_resource::ExtractResource},
};

use crate::session::XrTracker;

#[derive(SystemSet, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct XrSpaceSyncSet;

/// Any Spaces will be invalid after the owning session exits
#[repr(transparent)]
#[derive(Component, Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug, ExtractComponent)]
#[require(XrSpaceLocationFlags, Transform, Visibility, XrTracker)]
pub struct XrSpace(u64);

#[derive(Component, Clone, Copy, Reflect, Debug, ExtractComponent, Default)]
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

#[derive(Event, Clone, Copy, Deref, DerefMut)]
pub struct XrDestroySpace(pub XrSpace);

#[repr(transparent)]
#[derive(
    Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug, Component, Deref, DerefMut, ExtractComponent,
)]
pub struct XrReferenceSpace(pub XrSpace);

#[repr(transparent)]
#[derive(
    Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug, Resource, Deref, DerefMut, ExtractResource,
)]
pub struct XrPrimaryReferenceSpace(pub XrReferenceSpace);

#[derive(
    Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug, Component, ExtractComponent, Default,
)]
pub struct XrSpaceLocationFlags {
    pub position_tracked: bool,
    pub rotation_tracked: bool,
}

#[derive(
    Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug, Component, ExtractComponent, Default,
)]
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
