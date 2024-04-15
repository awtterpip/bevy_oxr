#[cfg(not(target_family = "wasm"))]
pub mod oxr;
#[cfg(target_family = "wasm")]
pub mod webxr;

#[cfg(not(target_family = "wasm"))]
pub use oxr::add_xr_plugins;
