use std::ops::Deref;
use std::rc::Rc;

use crate::prelude::*;

#[derive(Clone)]
pub struct Entry(Rc<dyn EntryTrait>);

#[derive(Clone)]
pub struct Instance(Rc<dyn InstanceTrait>);

#[derive(Clone)]
pub struct Session(Rc<dyn SessionTrait>);

#[derive(Clone)]
pub struct Input(Rc<dyn InputTrait>);

#[derive(Clone)]
pub struct Action<A: ActionType>(A::Inner);

impl Deref for Entry {
    type Target = dyn EntryTrait;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Deref for Instance {
    type Target = dyn InstanceTrait;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Deref for Session {
    type Target = dyn SessionTrait;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl Deref for Input {
    type Target = dyn InputTrait;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<O, A> Deref for Action<A>
where
    A: ActionType,
    A::Inner: Deref<Target = O>,
{
    type Target = O;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T: EntryTrait + 'static> From<T> for Entry {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: InstanceTrait + 'static> From<T> for Instance {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: SessionTrait + 'static> From<T> for Session {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}

impl<T: InputTrait + 'static> From<T> for Input {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
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

impl<T: ActionInputTrait<bool> + 'static> From<T> for Action<bool> {
    fn from(value: T) -> Self {
        Self(Rc::new(value))
    }
}
