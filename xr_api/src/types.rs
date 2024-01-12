use std::rc::Rc;

use glam::{Quat, Vec3};

use crate::api::Action;
use crate::api_traits::{ActionInputTrait, HapticTrait, InputTrait};
use crate::error::Result;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtensionSet {
    pub vulkan: bool,
}

pub struct SessionCreateInfo {
    /// preferred texture format
    pub texture_format: wgpu::TextureFormat,
}

pub struct Bindings {}

/// THIS IS NOT COMPLETE, im not sure how i am going to index actions currently.
#[derive(Clone, Copy, PartialEq)]
pub struct ActionPath {
    pub device: DevicePath,
    pub subpath: SubPath,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DevicePath {
    LeftHand,
    RightHand,
    Head,
    Gamepad,
    Treadmill,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SubPath {
    A,
    B,
    X,
    Y,
    Start,
    Home,
    End,
    Select,
    Joystick,
    Trigger,
    Squeeze,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Handedness {
    Left,
    Right,
    None,
}

pub struct Haptic;

pub struct Pose {
    pub translation: Vec3,
    pub rotation: Quat,
}

pub trait ActionType: Sized {
    type Inner: ?Sized;

    fn get(input: &dyn InputTrait, path: ActionPath) -> Result<Action<Self>>;
}

impl ActionType for Haptic {
    type Inner = dyn HapticTrait;

    fn get(input: &dyn InputTrait, path: ActionPath) -> Result<Action<Self>> {
        input.get_haptics(path)
    }
}

impl ActionType for Pose {
    type Inner = dyn ActionInputTrait<Pose>;

    fn get(input: &dyn InputTrait, path: ActionPath) -> Result<Action<Self>> {
        input.get_pose(path)
    }
}

impl ActionType for f32 {
    type Inner = dyn ActionInputTrait<f32>;

    fn get(input: &dyn InputTrait, path: ActionPath) -> Result<Action<Self>> {
        input.get_float(path)
    }
}

impl ActionType for bool {
    type Inner = dyn ActionInputTrait<bool>;

    fn get(input: &dyn InputTrait, path: ActionPath) -> Result<Action<Self>> {
        input.get_bool(path)
    }
}
