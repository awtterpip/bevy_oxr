mod extensions;
pub mod graphics;
mod resources;
pub mod types;

pub use resources::*;
pub use types::*;

use bevy::app::{App, Plugin};

pub fn xr_entry() -> Result<XrEntry> {
    #[cfg(windows)]
    let entry = openxr::Entry::linked();
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(entry.into())
}

fn init_xr() -> Result<()> {
    let entry = xr_entry()?;
    let instance = entry.create_instance(
        AppInfo::default(),
        XrExtensions::default(),
        GraphicsBackend::Vulkan(()),
    )?;
    let system_id = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

    instance.create_session(system_id)?;
    Ok(())
}

pub struct XrInitPlugin;

impl Plugin for XrInitPlugin {
    fn build(&self, app: &mut App) {
        let entry = xr_entry();
        todo!()
    }
}
