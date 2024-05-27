use std::{mem, ptr};

use bevy::prelude::*;
use bevy_xr::session::session_available;
use openxr::sys;

use crate::{
    openxr::exts::OxrEnabledExtensions,
    session_create_info_chain::{
        AdditionalSessionCreateInfo, AsAdditionalSessionCreateInfo, OxrSessionCreateInfoChain,
    },
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
        chain.push(OxrSessionCreateInfoOverlay::default());
    }
}
pub struct OxrSessionCreateInfoOverlay {
    inner: sys::SessionCreateInfoOverlayEXTX,
}
impl OxrSessionCreateInfoOverlay {
    pub const fn new(
        flags: openxr::OverlaySessionCreateFlagsEXTX,
        session_layers_placement: u32,
    ) -> Self {
        Self {
            inner: sys::SessionCreateInfoOverlayEXTX {
                ty: sys::SessionCreateInfoOverlayEXTX::TYPE,
                next: ptr::null(),
                create_flags: flags,
                session_layers_placement,
            },
        }
    }
}
impl Default for OxrSessionCreateInfoOverlay {
    fn default() -> Self {
        Self::new(openxr::OverlaySessionCreateFlagsEXTX::EMPTY, 0)
    }
}

impl AsAdditionalSessionCreateInfo for OxrSessionCreateInfoOverlay {
    fn header(&self) -> &AdditionalSessionCreateInfo {
        unsafe { mem::transmute(&self.inner) }
    }

    fn set_next(&mut self, next: &AdditionalSessionCreateInfo) {
        self.inner.next = next as *const _ as *const _;
    }
    fn clear_next(&mut self) {
        self.inner.next = ptr::null();
    }
}
