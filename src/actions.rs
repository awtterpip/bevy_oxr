use std::{borrow::Cow, marker::PhantomData};

use bevy::prelude::*;

pub use crate::action_paths::*;

#[derive(Event)]
pub struct XrCreateActionSet {
    pub handle: Handle<XrActionSet>,
    pub name: String,
}

pub struct XrAction<'a, T: ActionType> {
    pub name: Cow<'a, str>,
    pub pretty_name: Cow<'a, str>,
    pub action_set: Handle<XrActionSet>,
    _marker: PhantomData<T>,
}

#[derive(TypePath, Asset)]
pub struct XrActionSet {
    pub name: String,
}
