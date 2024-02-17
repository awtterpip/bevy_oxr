pub mod extensions;
pub mod init;
mod resources;
pub mod types;

pub use resources::*;

use bevy::app::{App, Plugin};

pub struct XrInitPlugin;

impl Plugin for XrInitPlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}
