use std::marker::PhantomData;

use crate::prelude::ActionType;

#[derive(Clone, Copy)]
pub struct ActionPath<P: InputComponent> {
    pub(crate) input: InputId,
    pub(crate) comp: PathComponent,
    _data: PhantomData<P>,
}

#[derive(Clone, Copy)]
pub struct UntypedActionPath {
    pub(crate) input: InputId,
    pub(crate) comp: PathComponent,
}

impl<P: InputComponent> From<ActionPath<P>> for UntypedActionPath {
    fn from(value: ActionPath<P>) -> Self {
        value.untyped()
    }
}

impl<P: InputComponent> ActionPath<P> {
    const fn new(input: InputId, comp: PathComponent) -> Self {
        Self {
            input,
            comp,
            //            hand,
            _data: PhantomData,
        }
    }

    pub fn untyped(self) -> UntypedActionPath {
        UntypedActionPath {
            input: self.input,
            comp: self.comp,
            //            hand: self.hand,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum PathComponent {
    Click,
    Touch,
    Value,
    X,
    Y,
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

pub struct X;

impl X {
    const COMP: PathComponent = PathComponent::X;
}

pub struct Y;

impl Y {
    const COMP: PathComponent = PathComponent::Y;
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
    type PathType = f32;
}

impl InputComponent for X {
    type PathType = f32;
}

impl InputComponent for Y {
    type PathType = f32;
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
                        $(#[$comp_name_handed_meta:meta])*
                        $comp_name_handed:ident
                    ),*
                    $(,)?
                }
            )*
        }
        $(
            $(#[$dev_path_meta:meta])*
            $dev_path:ident {
                $(
                    $(#[$inner_meta:meta])*
                    $inner:ident {
                        $(
                            $(#[$comp_name_meta:meta])*
                            $comp_name:ident
                        ),*
                        $(,)?
                    }
                )*
            }
        )*
    ) => {
        paste::paste! {
            const LEFT: bool = true;
            const RIGHT: bool = false;

            $(
                #[$id_meta]
            )*
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub(crate) enum $id {
                Left(Handed),
                Right(Handed),
                $(
                    $(
                        #[$dev_path_meta]
                    )*
                    [<$dev_path:camel>](input::$dev_path::[<$dev_path:camel>]),
                )*
            }

            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            pub(crate) enum Handed {
                $(
                    $(
                        #[$inner_handed_meta]
                    )*
                    $inner_handed,
                )*
            }

            pub mod input {
                use super::*;

                pub(crate) mod private {
                    $(
                        $(
                            #[$inner_handed_meta]
                        )*
                        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                        pub struct $inner_handed<const HAND: bool>;
                    )*
                }

                pub mod hand_left {
                    use super::*;

                    $(
                        $(
                            #[$inner_handed_meta]
                        )*
                        pub type $inner_handed = private::$inner_handed<LEFT>;

                        impl $inner_handed {
                            $(
                                $(
                                    #[$comp_name_handed_meta]
                                )*
                                pub const [<$comp_name_handed:snake:upper>]: ActionPath<$comp_name_handed> = ActionPath::<$comp_name_handed>::new($id::Left(Handed::$inner_handed), $comp_name_handed::COMP);
                            )*
                        }
                    )*
                }

                pub mod hand_right {
                    use super::*;

                    $(
                        $(
                            #[$inner_handed_meta]
                        )*
                        pub type $inner_handed = private::$inner_handed<RIGHT>;

                        impl $inner_handed {
                            $(
                                $(
                                    #[$comp_name_handed_meta]
                                )*
                                pub const [<$comp_name_handed:snake:upper>]: ActionPath<$comp_name_handed> = ActionPath::<$comp_name_handed>::new($id::Right(Handed::$inner_handed), $comp_name_handed::COMP);
                            )*
                        }
                    )*
                }

                $(
                    $(
                        #[$dev_path_meta]
                    )*
                    pub mod $dev_path {
                        use super::*;

                        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                        pub(crate) enum [<$dev_path:camel>] {
                            $(
                                $(
                                    #[$inner_meta]
                                )*
                                $inner,
                            )*
                        }

                        $(
                            $(
                                #[$inner_meta]
                            )*
                            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
                            pub struct $inner;

                            $(
                                #[$inner_meta]
                            )*
                            impl $inner {
                                $(
                                    $(
                                        #[$comp_name_meta]
                                    )*
                                    pub const [<$comp_name:snake:upper>]: ActionPath<$comp_name> = ActionPath::<$comp_name>::new($id::[<$dev_path:camel>]([<$dev_path:camel>]::$inner), $comp_name::COMP);
                                )*
                            }
                        )*
                    }
                )*
            }
        }
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
            X,
            Y,
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
        Output {
            Haptic,
        }
    }
    head {
        VolumeUp {
            Click,
        }
        VolumeDown {
            Click,
        }
        MuteMic {
            Click,
        }
    }
}
