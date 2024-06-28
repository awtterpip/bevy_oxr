use std::{mem, ptr};

use bevy::prelude::*;
use bevy_xr::session::session_available;
use openxr::sys;

use crate::{
    next_chain::{OxrNextChainStructBase, OxrNextChainStructProvider},
    openxr::exts::OxrEnabledExtensions,
    session::{OxrSessionCreateNextChain, OxrSessionCreateNextProvider},
};

pub struct OxrOverlayPlugin;

impl Plugin for OxrOverlayPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<OxrOverlaySessionEvent>();
        app.init_resource::<OxrOverlaySettings>();
        app.add_systems(First, add_overlay_info_to_chain.run_if(session_available));
    }
}

#[derive(Resource)]
pub struct OxrOverlaySettings {
    pub session_layer_placement: u32,
    pub flags: openxr::OverlaySessionCreateFlagsEXTX,
}

impl Default for OxrOverlaySettings {
    fn default() -> Self {
        OxrOverlaySettings {
            session_layer_placement: 0,
            flags: openxr::OverlaySessionCreateFlagsEXTX::EMPTY,
        }
    }
}

fn add_overlay_info_to_chain(
    mut chain: NonSendMut<OxrSessionCreateNextChain>,
    exts: Res<OxrEnabledExtensions>,
    settings: Res<OxrOverlaySettings>,
) {
    if exts.other.contains(&"XR_EXTX_overlay\0".to_string()) {
        chain.push(OxrSessionCreateInfoOverlay::new(
            settings.flags,
            settings.session_layer_placement,
        ));
    }
}

#[derive(Event, Clone, Copy, Debug)]
pub enum OxrOverlaySessionEvent {
    MainSessionVisibilityChanged {
        visible: bool,
        flags: openxr::OverlayMainSessionFlagsEXTX,
    },
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

impl OxrNextChainStructProvider for OxrSessionCreateInfoOverlay {
    fn header(&self) -> &OxrNextChainStructBase {
        unsafe { mem::transmute(&self.inner) }
    }

    fn set_next(&mut self, next: &OxrNextChainStructBase) {
        self.inner.next = next as *const _ as *const _;
    }
    fn clear_next(&mut self) {
        self.inner.next = ptr::null();
    }
}

impl OxrSessionCreateNextProvider for OxrSessionCreateInfoOverlay {}
