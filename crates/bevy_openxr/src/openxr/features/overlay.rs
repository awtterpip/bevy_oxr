use bevy::prelude::*;
use bevy_xr::session::session_available;

use crate::{
    openxr::exts::OxrEnabledExtensions,
    resources::OxrInstance,
    session_create_info_builder::{OxrSessionCreateInfoChain, OxrSessionCreateInfoOverlay},
};
#[derive(Default)]
pub struct OxrOverlayPlugin;
impl Plugin for OxrOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            update_chain.run_if(session_available.and_then(run_once())),
        );
    }
}

#[derive(Resource, Clone, Copy, Default)]
pub struct OxrOverlayPriority(pub u32);
fn update_chain(
    mut chain: NonSendMut<OxrSessionCreateInfoChain>,
    extensions: Res<OxrEnabledExtensions>,
) {
    if extensions.other.contains(&"XR_EXTX_overlay\0".to_string()) {
        chain.push(OxrSessionCreateInfoOverlay::default().to_openxr_info());
    }
}
