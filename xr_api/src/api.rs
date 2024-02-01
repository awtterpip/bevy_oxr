use std::ops::Deref;
use std::rc::Rc;

use glam::Vec2;

use crate::prelude::*;

/// Entry point to the API
///
/// To see methods available for this struct, refer to [EntryTrait]
#[derive(Clone)]
pub struct Entry(Rc<dyn EntryTrait>);

impl Entry {
    /// Constructs a new Xr entry
    pub fn new() -> Self {
        #[cfg(target_family = "wasm")]
        return crate::backend::webxr::WebXrEntry::new().into();
        #[cfg(not(target_family = "wasm"))]
        return crate::backend::oxr::OXrEntry::new().into();
    }
}

/// Represents an intent to start a session with requested extensions.
///
/// To see methods available for this struct, refer to [InstanceTrait]
#[derive(Clone)]
pub struct Instance(Rc<dyn InstanceTrait>);

/// Represents a running XR application.
///
/// To see methods available for this struct, refer to [SessionTrait]
#[derive(Clone)]
pub struct Session(Rc<dyn SessionTrait>);

/// A view of one eye. Used to retrieve render data such as texture views and projection matrices.
///
/// To see methods available for this struct, refer to [ViewTrait]
#[derive(Clone)]
pub struct View(Rc<dyn ViewTrait>);

/// Represents all XR input sources.
///
/// To see methods available for this struct, refer to [InputTrait]
#[derive(Clone)]
pub struct Input(Rc<dyn InputTrait>);

/// Represents an XR Action. Can be used to retrieve input values or trigger output devices such as haptics.
///
/// The methods available to this struct are dependent upon the action type. For input values, use `.get()` to retrieve the values.
/// For haptics, please refer to [HapticTrait]
#[derive(Clone)]
pub struct Action<A: ActionType>(Rc<A::Inner>);

macro_rules! impl_api {
    ($($t:ty, $trait:ident; )*) => {
        $(
            impl std::ops::Deref for $t {
                type Target = dyn $trait;

                fn deref(&self) -> &Self::Target {
                    &*self.0
                }
            }

            impl<T: $trait + 'static> From<T> for $t {
                fn from(value: T) -> Self {
                    Self(Rc::new(value))
                }
            }
        )*

    };
}

impl_api! {
    Entry, EntryTrait;
    Instance, InstanceTrait;
    Session, SessionTrait;
    View, ViewTrait;
    Input, InputTrait;
}

impl<A: ActionType> Deref for Action<A> {
    type Target = A::Inner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: HapticTrait + 'static> From<T> for Action<Haptic> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ActionInputTrait<f32> + 'static> From<T> for Action<f32> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ActionInputTrait<Pose> + 'static> From<T> for Action<Pose> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ActionInputTrait<Vec2> + 'static> From<T> for Action<Vec2> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: ActionInputTrait<bool> + 'static> From<T> for Action<bool> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}
