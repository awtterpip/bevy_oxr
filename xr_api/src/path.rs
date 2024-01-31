use std::marker::PhantomData;

use glam::Vec2;

use crate::prelude::ActionType;

pub struct ActionPath<P: InputComponent> {
    pub(crate) input: InputId,
    pub(crate) comp: PathComponent,
    pub(crate) hand: Option<Hand>,
    _data: PhantomData<P>,
}

pub struct UntypedActionPath {
    pub(crate) input: InputId,
    pub(crate) comp: PathComponent,
    pub(crate) hand: Option<Hand>,
}

impl<P: InputComponent> From<ActionPath<P>> for UntypedActionPath {
    fn from(value: ActionPath<P>) -> Self {
        value.untyped()
    }
}

impl<P: InputComponent> ActionPath<P> {
    const fn new(input: InputId, comp: PathComponent, hand: Option<Hand>) -> Self {
        Self {
            input,
            comp,
            hand,
            _data: PhantomData,
        }
    }

    pub fn untyped(self) -> UntypedActionPath {
        UntypedActionPath {
            input: self.input,
            comp: self.comp,
            hand: self.hand,
        }
    }
}

pub(crate) enum Hand {
    Left,
    Right,
}

pub(crate) enum PathComponent {
    Click,
    Touch,
    Value,
    Axes,
    Pose,
    Haptic,
}

pub struct Click;

impl Click {
    const COMP: PathComponent = PathComponent::Click;
}

pub struct Touch;

impl Touch {
    const COMP: PathComponent = PathComponent::Touch;
}

pub struct Value;

impl Value {
    const COMP: PathComponent = PathComponent::Value;
}

pub struct Axes;

impl Axes {
    const COMP: PathComponent = PathComponent::Axes;
}

pub struct Pose;

impl Pose {
    const COMP: PathComponent = PathComponent::Pose;
}

pub struct Haptic;

impl Haptic {
    const COMP: PathComponent = PathComponent::Haptic;
}

pub trait InputComponent {
    type PathType: ActionType;
}

impl InputComponent for Click {
    type PathType = bool;
}

impl InputComponent for Touch {
    type PathType = bool;
}

impl InputComponent for Value {
    type PathType = bool;
}

impl InputComponent for Axes {
    type PathType = Vec2;
}

impl InputComponent for Pose {
    type PathType = crate::types::Pose;
}

impl InputComponent for Haptic {
    type PathType = crate::types::Haptic;
}

macro_rules! input_ids {
    (
        $(#[$id_meta:meta])*
        $id:ident;
        handed {
            $(
                $(#[$inner_handed_meta:meta])*
                $inner_handed:ident {
                    $(
                        $comp_name_handed:ident,
                    )*
                }
            )*
        }
        $(
            $(#[$inner_meta:meta])*
            $inner:ident {
                $(
                    $comp_name:ident,
                )*
            }
        )*
    ) => {
        $(
            #[$id_meta]
        )*
        paste::paste! {
            pub(crate) enum $id {
                $(
                    $inner,
                )*
                $(
                    [<$inner_handed Left>],
                    [<$inner_handed Right>],
                )*
            }
        }

        pub mod left {
            const LEFT: bool = true;
            $(
                pub type $inner_handed = super::$inner_handed<LEFT>;
            )*
        }

        pub mod right {
            const RIGHT: bool = false;
            $(
                pub type $inner_handed = super::$inner_handed<RIGHT>;
            )*
        }

        $(
            $(
                #[$inner_handed_meta]
            )*
            pub struct $inner_handed<const HAND: bool>;
            impl $inner_handed<true> {
                paste::paste! {
                    $(
                        pub const [<$comp_name_handed:snake:upper>]: ActionPath<$comp_name_handed> = ActionPath::<$comp_name_handed>::new($id::[<$inner_handed Left>], $comp_name_handed::COMP, Some(Hand::Left));
                    )*
                }
            }
            impl $inner_handed<false> {
                paste::paste! {
                    $(
                        pub const [<$comp_name_handed:snake:upper>]: ActionPath<$comp_name_handed> = ActionPath::<$comp_name_handed>::new($id::[<$inner_handed Right>], $comp_name_handed::COMP, Some(Hand::Right));
                    )*
                }
            }

        )*

        $(
            $(
                #[$inner_meta]
            )*
            pub struct $inner;

            impl $inner {
                paste::paste! {
                    $(
                        pub const [<$comp_name:snake:upper>]: ActionPath<$comp_name> = ActionPath::<$comp_name>::new($id::$inner, $comp_name::COMP, None);
                    )*
                }
            }
        )*
    };
}

input_ids! {
    InputId;
    handed {
        PrimaryButton {
            Click,
            Touch,
        }
        SecondaryButton {
            Click,
            Touch,
        }
        Select {
            Click,
        }
        Menu {
            Click,
        }
        Thumbstick {
            Axes,
            Click,
            Touch,
        }
        Trigger {
            Touch,
            Click,
        }
        Grip {
            Click,
            Value,
            Pose,
        }
    }
}
