#[cfg(not(target_family = "wasm"))]
pub(crate) mod graphics;
#[cfg(not(target_family = "wasm"))]
mod oxr;
#[cfg(not(target_family = "wasm"))]
pub(crate) mod oxr_utils;
pub mod traits;
#[cfg(target_family = "wasm")]
pub(crate) mod web_utils;
#[cfg(target_family = "wasm")]
mod webxr;

#[cfg(not(target_family = "wasm"))]
pub use oxr::*;
#[cfg(target_family = "wasm")]
pub use webxr::*;

macro_rules! xr_inner {
    ($res:ty, $oxr:ty, $webxr:ty) => {
        paste::paste! {
            pub enum [<$res Inner>] {
                #[cfg(not(target_family = "wasm"))]
                OpenXR($oxr),
                #[cfg(target_family = "wasm")]
                WebXR($webxr),
            }

            #[cfg(not(target_family = "wasm"))]
            impl From<$oxr> for $res {
                fn from(value: $oxr) -> $res {
                    $res(std::rc::Rc::new([<$res Inner>]::OpenXR(value)))
                }
            }

            #[cfg(target_family = "wasm")]
            impl From<$webxr> for $res {
                fn from(value: $webxr) -> $res {
                    $res(std::rc::Rc::new([<$res Inner>]::WebXR(value)))
                }
            }
        }
    };
}

use crate::resources::*;

xr_inner!(XrEntry, OXrEntry, WebXrEntry);
xr_inner!(XrInstance, OXrInstance, WebXrInstance);
xr_inner!(XrSession, OXrSession, WebXrSession);
xr_inner!(XrInput, OXrAction, WebXrAction);
xr_inner!(XrController, OXrController, WebXrActionSet);
xr_inner!(XrActionSpace, OXrActionSpace, WebXrActionSpace);
xr_inner!(XrReferenceSpace, OXrReferenceSpace, WebXrReferenceSpace);
