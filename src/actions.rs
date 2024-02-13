pub mod oculus_touch;

mod private {
    use bevy::math::Vec2;

    use crate::types::{Haptic, Pose};

    pub trait Sealed {}

    impl Sealed for bool {}
    impl Sealed for f32 {}
    impl Sealed for Vec2 {}
    impl Sealed for Pose {}
    impl Sealed for Haptic {}
}

use std::borrow::Cow;
use std::marker::PhantomData;

pub trait ActionType: private::Sealed {}

impl<T: private::Sealed> ActionType for T {}

pub trait ActionPathTrait {
    type PathType: ActionType;
    fn path(&self) -> Cow<'_, str>;
    fn name(&self) -> Cow<'_, str>;
}

pub struct ActionPath<T: ActionType> {
    path: &'static str,
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T: ActionType> ActionPathTrait for ActionPath<T> {
    type PathType = T;

    fn path(&self) -> Cow<'_, str> {
        self.path.into()
    }

    fn name(&self) -> Cow<'_, str> {
        self.name.into()
    }
}

macro_rules! actions {
    // create path struct
    (
        $($subpath:literal),*
        $id:ident {
            path: $path:literal;
        }
    ) => {};

    // handle action path attrs
    (
        $($subpath:literal),*
        $id:ident {
            path: $path:literal;
            name: $name:literal;
            path_type: $path_type:ty;
        }
    ) => {
        paste::paste! {
            pub const [<$id:snake:upper>]: crate::actions::ActionPath<$path_type> = crate::actions::ActionPath {
                path: concat!($($subpath,)* $path),
                name: $name,
                _marker: std::marker::PhantomData,
            };
        }
    };

    // handle action path attrs
    (
        $($subpath:literal),*
        $id:ident {
            path: $path:literal;
            name: $name:literal;
            path_type: $path_type:ty;
            $($children:tt)*
        }
    ) => {
        crate::path::actions! {
            $($subpath),*
            $id {
                path: $path;
                name: $name;
                path_type: $path_type;
            }
        }

        crate::path::actions! {
            $($subpath),*
            $id {
                path: $path;
                $($children)*
            }
        }
    };

    // handle children
    (
        $($subpath:literal),*
        $id:ident {
            path: $path:literal;
            $($children:tt)*
        }
    ) => {
        pub mod $id {
            crate::actions::actions! {
                $($subpath,)* $path
                $($children)*
            }
        }
    };

    // handle siblings
    (
        $($subpath:literal),*
        $id:ident {
            path: $path:literal;
            $($attrs:tt)*
        }
        $($siblings:tt)*
    ) => {
        crate::actions::actions! {
            $($subpath),*
            $id {
                path: $path;
                $($attrs)*
            }
        }
        crate::actions::actions! {
            $($subpath),*
            $($siblings)*
        }
    };
}

pub(crate) use actions;
