use bevy::prelude::*;
use openxr::sys::SystemPassthroughProperties2FB;
use openxr::PassthroughCapabilityFlagsFB;

use crate::resources::*;
use crate::types::*;

pub struct OxrPassthroughPlugin;

impl Plugin for OxrPassthroughPlugin {
    fn build(&self, _app: &mut App) {
        todo!()
    }
}

pub fn create_passthrough(
    session: &OxrSession,
    flags: openxr::PassthroughFlagsFB,
    purpose: openxr::PassthroughLayerPurposeFB,
) -> Result<(OxrPassthrough, OxrPassthroughLayer)> {
    let passthrough = session.create_passthrough(flags)?;

    let passthrough_layer = session.create_passthrough_layer(&passthrough, purpose)?;

    Ok((passthrough, passthrough_layer))
}

#[inline]
pub fn supports_passthrough(instance: &OxrInstance, system: OxrSystemId) -> Result<bool> {
    unsafe {
        let mut hand = openxr::sys::SystemPassthroughProperties2FB {
            ty: SystemPassthroughProperties2FB::TYPE,
            next: std::ptr::null(),
            capabilities: PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY,
        };
        let mut p = openxr::sys::SystemProperties::out(&mut hand as *mut _ as _);
        cvt((instance.fp().get_system_properties)(
            instance.as_raw(),
            system.0,
            p.as_mut_ptr(),
        ))?;
        bevy::log::info!(
            "From supports_passthrough: Passthrough capabilities: {:?}",
            hand.capabilities
        );
        Ok(
            (hand.capabilities & PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY)
                == PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY,
        )
    }
}

fn cvt(x: openxr::sys::Result) -> openxr::Result<openxr::sys::Result> {
    if x.into_raw() >= 0 {
        Ok(x)
    } else {
        Err(x)
    }
}
