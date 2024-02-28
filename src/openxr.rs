mod extensions;
pub mod graphics;
mod resources;
pub mod types;

pub use resources::*;
pub use types::*;

use bevy::app::{App, Plugin};
use bevy::log::error;

pub fn xr_entry() -> Result<XrEntry> {
    #[cfg(windows)]
    let entry = openxr::Entry::linked();
    #[cfg(not(windows))]
    let entry = unsafe { openxr::Entry::load()? };
    Ok(XrEntry(entry))
}

pub struct XrInitPlugin {
    /// Information about the app this is being used to build.
    pub app_info: AppInfo,
    /// Extensions wanted for this session.
    // This should preferably be changed into a simpler list of features wanted that this crate supports. i.e. hand tracking
    pub exts: XrExtensions,
    /// List of backends the openxr session can use. If [None], pick the first available backend.
    pub backends: Option<Vec<GraphicsBackend>>,
}

impl Plugin for XrInitPlugin {
    fn build(&self, app: &mut App) {
        init_xr(self, app).unwrap();
        todo!()
    }
}

fn init_xr(config: &XrInitPlugin, _app: &mut App) -> Result<()> {
    let entry = xr_entry()?;

    let available_exts = entry.enumerate_extensions()?;

    for ext in available_exts.unavailable_exts(&config.exts) {
        error!(
            "Extension \"{ext}\" not available in the current openxr runtime. Disabling extension."
        );
    }

    let available_backends = GraphicsBackend::available_backends(&available_exts);

    // Backend selection
    let backend = if let Some(wanted_backends) = &config.backends {
        let mut backend = None;
        for wanted_backend in wanted_backends {
            if available_backends.contains(wanted_backend) {
                backend = Some(*wanted_backend);
                break;
            }
        }
        backend
    } else {
        available_backends.first().copied()
    }
    .ok_or(XrError::NoAvailableBackend)?;

    let exts = config.exts.clone() & available_exts;

    let instance = entry.create_instance(config.app_info.clone(), exts, backend)?;
    let _system_id = instance.system(openxr::FormFactor::HEAD_MOUNTED_DISPLAY)?;

    //instance.create_session(system_id)?;
    Ok(())
}
