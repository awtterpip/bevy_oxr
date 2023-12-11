use std::rc::Rc;

use bevy::prelude::{Deref, DerefMut};

use crate::backend;

macro_rules! xr_resources {
    (
        $(
            $(#[$attr:meta])*
            $name:ident;
        )*
    ) => {
        paste::paste! {
            $(
                $(#[$attr])*
                #[derive(Clone, Deref, DerefMut)]
                pub struct $name(pub(crate) Rc<backend::[<$name Inner>]>);
            )*
        }
    };
}

xr_resources! {
    XrEntry;
    XrInstance;
    XrSession;
    XrInput;
    XrController;
    XrActionSpace;
    XrReferenceSpace;
}
