#[cfg(target_family = "wasm")]
mod webxr;

#[cfg(target_family = "wasm")]
pub use webxr::*;
