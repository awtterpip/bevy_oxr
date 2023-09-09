use openxr::{Action, ActionTy};

pub struct Touchable<T: ActionTy> {
    pub inner: Action<T>,
    pub touch: Action<bool>,
}
pub struct Handed<T> {
    pub left: T,
    pub right: T,
}
#[derive(Copy, Clone)]
pub enum XrControllerType {
    OculusTouch,
}
