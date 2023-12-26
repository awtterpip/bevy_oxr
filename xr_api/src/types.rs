use std::rc::Rc;

use crate::api_traits::{ActionInputTrait, HapticTrait};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ExtensionSet {}

pub enum SessionCreateInfo {}

pub struct Bindings {}

#[derive(Clone, Copy, PartialEq)]
pub struct ActionId {
    pub handedness: Handedness,
    pub device: Device,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Handedness {
    Left,
    Right,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Device {
    Controller,
}

pub struct Haptic;
pub struct Pose;

pub trait ActionType {
    type Inner;
}

impl ActionType for Haptic {
    type Inner = Rc<dyn HapticTrait>;
}

impl ActionType for Pose {
    type Inner = Rc<dyn ActionInputTrait<Pose>>;
}

impl ActionType for f32 {
    type Inner = Rc<dyn ActionInputTrait<f32>>;
}

impl ActionType for bool {
    type Inner = Rc<dyn ActionInputTrait<bool>>;
}
