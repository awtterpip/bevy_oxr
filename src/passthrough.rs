use bevy::prelude::*;
use std::ptr;

use openxr::{self as xr, sys::PassthroughCreateInfoFB, PassthroughFlagsFB, StructureType};
use xr::{
    sys::{
        loader::ApiLayerCreateInfo, PassthroughFB, PassthroughLayerCreateInfoFB,
        PassthroughLayerFB, SystemPassthroughProperties2FB,
    },
    PassthroughCapabilityFlagsFB, PassthroughLayerPurposeFB,
};

use crate::{resources::XrInstance, xr_init::XrRenderData};
#[derive(Resource, Clone)]
pub struct XrPassthroughLayer(pub PassthroughLayerFB);
#[derive(Resource, Clone)]
pub struct XrPassthrough(pub PassthroughFB);

pub fn start_passthrough(data: &XrRenderData) -> (XrPassthroughLayer, XrPassthrough) {
    let p = data.xr_instance.exts().fb_passthrough.unwrap();
    let pass = unsafe {
        let passthrough = PassthroughCreateInfoFB {
            ty: StructureType::PASSTHROUGH_CREATE_INFO_FB,
            next: ptr::null(),
            flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
        };
        let mut pass = xr::sys::PassthroughFB::NULL;
        let w = (p.create_passthrough)(
            data.xr_session.as_raw(),
            &passthrough as *const _,
            &mut pass as *mut _,
        );
        match cvt(w) {
            Ok(_) => pass,
            Err(err) => panic!("unable to create passthrough: {}", err),
        }
    };
    // unsafe {
    //     (p.passthrough_start)(pass);
    // }
    let pass_layer = unsafe {
        let passthrough = PassthroughLayerCreateInfoFB {
            ty: PassthroughLayerCreateInfoFB::TYPE,
            next: ptr::null(),
            passthrough: pass,
            flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
            purpose: PassthroughLayerPurposeFB::RECONSTRUCTION,
        };
        let mut pass_layer = xr::sys::PassthroughLayerFB::NULL;
        let w = (p.create_passthrough_layer)(
            data.xr_session.as_raw(),
            &passthrough as *const _,
            &mut pass_layer as *mut _,
        );
        match cvt(w) {
            Ok(_) => pass_layer,
            Err(err) => panic!("unable to create passthrough layer: {}", err),
        }
    };
    unsafe {
        (p.passthrough_start)(pass);
        (p.passthrough_layer_resume)(pass_layer);
    }
    (XrPassthroughLayer(pass_layer), XrPassthrough(pass))
}
fn cvt(x: xr::sys::Result) -> xr::Result<xr::sys::Result> {
    if x.into_raw() >= 0 {
        Ok(x)
    } else {
        Err(x)
    }
}
#[inline]
pub fn supports_passthrough(instance: &XrInstance, system: xr::SystemId) -> xr::Result<bool> {
    unsafe {
        let mut hand = xr::sys::SystemPassthroughProperties2FB {
            ty: SystemPassthroughProperties2FB::TYPE,
            next: ptr::null(),
            capabilities: PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY,
        };
        let mut p = xr::sys::SystemProperties::out(&mut hand as *mut _ as _);
        cvt((instance.fp().get_system_properties)(
            instance.as_raw(),
            system,
            p.as_mut_ptr(),
        ))?;
        Ok(
            (hand.capabilities & PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY)
                == PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY,
        )
    }
}
