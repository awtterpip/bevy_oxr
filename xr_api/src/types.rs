use glam::{Quat, Vec2, Vec3};

use crate::api::Action;
use crate::api_traits::{ActionInputTrait, HapticTrait, InputTrait};
use crate::error::Result;
use crate::path::UntypedActionPath;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtensionSet {
    pub vulkan: bool,
}

pub struct SessionCreateInfo {
    /// preferred texture format
    pub texture_format: wgpu::TextureFormat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bindings {
    OculusTouch,
}

pub struct Haptic;

pub struct Pose {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub trait ActionType: Sized {
    type Inner: ?Sized;

    fn get(input: &dyn InputTrait, path: UntypedActionPath) -> Result<Action<Self>>;
}

impl ActionType for Haptic {
    type Inner = dyn HapticTrait;

    fn get(input: &dyn InputTrait, path: UntypedActionPath) -> Result<Action<Self>> {
        input.create_action_haptics(path)
    }
}

impl ActionType for Pose {
    type Inner = dyn ActionInputTrait<Pose>;

    fn get(input: &dyn InputTrait, path: UntypedActionPath) -> Result<Action<Self>> {
        input.create_action_pose(path)
    }
}

impl ActionType for f32 {
    type Inner = dyn ActionInputTrait<f32>;

    fn get(input: &dyn InputTrait, path: UntypedActionPath) -> Result<Action<Self>> {
        input.create_action_float(path)
    }
}

impl ActionType for bool {
    type Inner = dyn ActionInputTrait<bool>;

    fn get(input: &dyn InputTrait, path: UntypedActionPath) -> Result<Action<Self>> {
        input.create_action_bool(path)
    }
}

impl ActionType for Vec2 {
    type Inner = dyn ActionInputTrait<Vec2>;

    fn get(input: &dyn InputTrait, path: UntypedActionPath) -> Result<Action<Self>> {
        input.create_action_vec2(path)
    }
}
