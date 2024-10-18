use bevy::{
    ecs::component::StorageType,
    prelude::*,
    render::{extract_component::ExtractComponent, extract_resource::ExtractResource},
};

/// Any Spaces will be invalid after the owning session exits
#[repr(transparent)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Reflect, Debug, ExtractComponent)]
pub struct XrSpace(u64);

#[derive(Clone, Copy, Reflect, Debug, ExtractComponent, Default)]
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

impl Component for XrSpace {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            world
                .commands()
                .entity(entity)
                .insert(XrSpaceLocationFlags::default());
        });
    }
}
impl Component for XrVelocity {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            world
                .commands()
                .entity(entity)
                .insert(XrSpaceVelocityFlags::default());
        });
    }
}
