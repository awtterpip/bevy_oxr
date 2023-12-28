//! Abstracted API over WebXR and OpenXR
//!
//! This crate is intended to be used as a common API for cross platform projects. It was primarily
//! made for use in Bevy, but can be used elsewhere.
//!
//! To get started, create an [Entry] with [Entry](Entry#method.new)

mod api;
pub mod api_traits;
pub mod backend;
pub mod error;
pub mod types;

pub mod prelude {
    pub use super::api::*;
    pub use super::api_traits::*;
    pub use super::error::*;
    pub use super::types::*;
}

pub use api::*;
