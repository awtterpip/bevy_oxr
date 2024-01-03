use std::ptr::null;

use openxr::SystemId;

use crate::{
    resources::{XrInstance, XrPassthroughLayer},
    xr_init::XrRenderData,
};
use openxr as xr;
use xr::{
    sys::{
        PassthroughCreateInfoFB, PassthroughFB, PassthroughLayerCreateInfoFB, PassthroughLayerFB,
    },
    PassthroughFlagsFB, PassthroughLayerPurposeFB,
};

pub fn start_passthrough(render_data: &XrRenderData) -> (XrPassthroughLayer, PassthroughFB) {
    let instance = &render_data.xr_instance;
    let entry = instance.entry();
    let instance = instance.as_raw();
    let session = render_data.xr_session.as_raw();

    let passthrough_fb_vtable =
        unsafe { openxr::raw::PassthroughFB::load(entry, instance) }.unwrap();

    // Configuration for creating the passthrough feature
    let passthrough_create_info = PassthroughCreateInfoFB {
        ty: PassthroughCreateInfoFB::TYPE,
        next: null(),
        flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
    };

    let mut passthrough_feature = openxr::sys::PassthroughFB::NULL;
    let result = unsafe {
        (passthrough_fb_vtable.create_passthrough)(
            session,
            &passthrough_create_info,
            &mut passthrough_feature,
        )
    };

    if result != openxr::sys::Result::SUCCESS {
        panic!("Failed to start passthough layer:\n{result:?}");
    }

    let passthrough = PassthroughFB::NULL;

    let passthrough_layer_info = PassthroughLayerCreateInfoFB {
        ty: PassthroughLayerCreateInfoFB::TYPE,
        next: null(),
        passthrough: passthrough,
        flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
        purpose: PassthroughLayerPurposeFB::RECONSTRUCTION,
    };

    let mut passthrough_layer_fb = PassthroughLayerFB::NULL;
    let result = unsafe {
        (passthrough_fb_vtable.create_passthrough_layer)(
            session,
            &passthrough_layer_info,
            &mut passthrough_layer_fb,
        )
    };

    if result != openxr::sys::Result::SUCCESS {
        panic!("Failed to create a passthough layer:\n{result:?}");
    }

    (XrPassthroughLayer::new(passthrough_layer_fb), passthrough_feature)
}

pub fn supports_passthrough(a: &XrInstance, b: SystemId) -> Result<bool, ()> {
    Ok(true)
}
