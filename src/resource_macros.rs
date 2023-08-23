#[macro_export]
macro_rules! xr_resource_wrapper {
    ($wrapper_type:ident, $xr_type:ty) => {
        #[derive(Clone, bevy::prelude::Resource)]
        pub struct $wrapper_type($xr_type);

        impl $wrapper_type {
            pub fn new(value: $xr_type) -> Self {
                Self(value)
            }
        }

        impl std::ops::Deref for $wrapper_type {
            type Target = $xr_type;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$xr_type> for $wrapper_type {
            fn from(value: $xr_type) -> Self {
                Self::new(value)
            }
        }
    }
}

#[macro_export]
macro_rules! xr_arc_resource_wrapper {
    ($wrapper_type:ident, $xr_type:ty) => {
        #[derive(Clone, bevy::prelude::Resource)]
        pub struct $wrapper_type(std::sync::Arc<$xr_type>);

        impl $wrapper_type {
            pub fn new(value: $xr_type) -> Self {
                Self(std::sync::Arc::new(value))
            }
        }

        impl std::ops::Deref for $wrapper_type {
            type Target = $xr_type;

            fn deref(&self) -> &Self::Target {
                self.0.as_ref()
            }
        }

        impl From<$xr_type> for $wrapper_type {
            fn from(value: $xr_type) -> Self {
                Self::new(value)
            }
        }
    }
}

pub use xr_resource_wrapper;
pub use xr_arc_resource_wrapper;