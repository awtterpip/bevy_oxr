use std::rc::Rc;

use crate::api::Action;
use crate::api_traits::{ActionInputTrait, HapticTrait, InputTrait};
use crate::error::Result;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtensionSet {}

pub enum SessionCreateInfo {}

pub struct Bindings {}

/// THIS IS NOT COMPLETE, im not sure how i am going to index actions currently.
#[derive(Clone, Copy, PartialEq)]
pub struct ActionId {
    pub handedness: Handedness,
    pub device: XrDevice,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Handedness {
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum XrDevice {
    Controller,
}

pub struct Haptic;
pub struct Pose;

pub trait ActionType: Sized {
    type Inner: ?Sized;

    fn get(input: &dyn InputTrait, path: ActionId) -> Result<Action<Self>>;
}

impl ActionType for Haptic {
    type Inner = dyn HapticTrait;

    fn get(input: &dyn InputTrait, path: ActionId) -> Result<Action<Self>> {
        input.get_haptics(path)
    }
}

impl ActionType for Pose {
    type Inner = dyn ActionInputTrait<Pose>;

    fn get(input: &dyn InputTrait, path: ActionId) -> Result<Action<Self>> {
        input.get_pose(path)
    }
}

impl ActionType for f32 {
    type Inner = dyn ActionInputTrait<f32>;

    fn get(input: &dyn InputTrait, path: ActionId) -> Result<Action<Self>> {
        input.get_float(path)
    }
}

impl ActionType for bool {
    type Inner = dyn ActionInputTrait<bool>;

    fn get(input: &dyn InputTrait, path: ActionId) -> Result<Action<Self>> {
        input.get_bool(path)
    }
}
