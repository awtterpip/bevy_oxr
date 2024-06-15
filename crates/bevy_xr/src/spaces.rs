use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, extract_resource::ExtractResource},
};

/// Any Spaces will be invalid after the owning session exits
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug)]
pub struct XrSpace(u64);

#[derive(Clone, Copy, PartialEq, Reflect, Debug, Component, ExtractComponent)]
pub struct XrSpatialTransform {
    pub space: XrSpace,
    pub offset: Transform,
}
impl XrSpatialTransform {
    pub const fn from_space(space: XrSpace) -> Self {
        Self { space, offset: Transform::IDENTITY }
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
