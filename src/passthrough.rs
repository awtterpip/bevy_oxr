use bevy::prelude::*;
use std::{marker::PhantomData, mem, ptr};

use openxr as xr;
use xr::{
    sys::{PassthroughFB, PassthroughLayerFB, Space, SystemPassthroughProperties2FB},
    CompositionLayerBase, CompositionLayerFlags, Graphics, PassthroughCapabilityFlagsFB,
};

use crate::resources::XrInstance;
use crate::resources::XrSession;
pub struct PassthroughLayer(pub xr::sys::PassthroughLayerFB);
pub struct Passthrough(pub xr::sys::PassthroughFB);
fn cvt(x: xr::sys::Result) -> xr::Result<xr::sys::Result> {
    if x.into_raw() >= 0 {
        Ok(x)
    } else {
        Err(x)
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub(crate) struct CompositionLayerPassthrough<'a, G: xr::Graphics> {
    inner: xr::sys::CompositionLayerPassthroughFB,
    _marker: PhantomData<&'a G>,
}
impl<'a, G: Graphics> std::ops::Deref for CompositionLayerPassthrough<'a, G> {
    type Target = CompositionLayerBase<'a, G>;
    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(&self.inner) }
    }
}

impl<'a, G: xr::Graphics> CompositionLayerPassthrough<'a, G> {
    pub(crate) fn from_xr_passthrough_layer(layer: &PassthroughLayer) -> Self {
        Self {
            inner: xr::sys::CompositionLayerPassthroughFB {
                ty: xr::sys::CompositionLayerPassthroughFB::TYPE,
                next: ptr::null(),
                flags: CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA,
                space: Space::NULL,
                layer_handle: layer.0,
            },
            _marker: PhantomData,
        }
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

#[inline]
pub fn start_passthrough(
    instance: &XrInstance,
    xr_session: &XrSession,
) -> xr::Result<(xr::sys::PassthroughFB, xr::sys::PassthroughLayerFB)> {
    unsafe {
        // Create feature
        let mut passthrough_feature = xr::sys::PassthroughFB::NULL;
        let mut passthrough_create_info = xr::sys::PassthroughCreateInfoFB {
            ty: SystemPassthroughProperties2FB::TYPE,
            next: ptr::null(),
            flags: xr::sys::PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
        };
        cvt(
            (instance.exts().fb_passthrough.unwrap().create_passthrough)(
                xr_session.as_raw(),
                &passthrough_create_info as *const _,
                &mut passthrough_feature as *mut _,
            ),
        );

        // Create layer
        let mut passthrough_layer = xr::sys::PassthroughLayerFB::NULL;
        let mut layer_create_info = xr::sys::PassthroughLayerCreateInfoFB {
            ty: xr::sys::StructureType::PASSTHROUGH_LAYER_CREATE_INFO_FB, // XR_TYPE_PASSTHROUGH_LAYER_CREATE_INFO_FB
            next: ptr::null(),
            passthrough: passthrough_feature, // XR_PASSTHROUGH_HANDLE
            flags: xr::sys::PassthroughFlagsFB::IS_RUNNING_AT_CREATION, // XR_PASSTHROUGH_IS_RUNNING_AT_CREATION_BIT_FB
            purpose: xr::sys::PassthroughLayerPurposeFB::RECONSTRUCTION, // XR_PASSTHROUGH_LAYER_PURPOSE_RECONSTRUCTION_FB
        };
        cvt((instance
            .exts()
            .fb_passthrough
            .unwrap()
            .create_passthrough_layer)(
            xr_session.as_raw(),
            &layer_create_info as *const _,
            &mut passthrough_layer as *mut _,
        ));
        // Start layer

        cvt((instance.exts().fb_passthrough.unwrap().passthrough_start)(
            passthrough_feature,
        ));

        Ok((passthrough_feature, passthrough_layer))
    }
}
