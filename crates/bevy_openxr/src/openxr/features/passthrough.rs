use bevy::prelude::*;
use openxr::sys::SystemPassthroughProperties2FB;
use openxr::PassthroughCapabilityFlagsFB;
use openxr::PassthroughFlagsFB;
use openxr::PassthroughLayerPurposeFB;

use crate::init::OxrSessionResourceCreator;
use crate::init::OxrSessionResourceCreatorsApp;
use crate::layer_builder::PassthroughLayer;
use crate::resources::*;
use crate::types::*;

pub struct OxrPassthroughPlugin;

impl Plugin for OxrPassthroughPlugin {
    fn build(&self, app: &mut App) {
        let resources = app
            .world
            .get_resource::<OxrInstance>()
            .and_then(|instance| {
                app.world
                    .get_resource::<OxrSystemId>()
                    .map(|system_id| (instance, system_id))
            });
        if resources.is_some_and(|(instance, system)| {
            supports_passthrough(instance, *system).is_ok_and(|s| s)
        }) {
            app.add_xr_resource_creator(OxrPassthroughCreator {
                flags: PassthroughFlagsFB::IS_RUNNING_AT_CREATION,
                purpose: PassthroughLayerPurposeFB::RECONSTRUCTION,
                layer: None,
                passthrough: None,
            });
        } else {
            error!("Passthrough is not supported with this runtime")
        }
    }
}

struct OxrPassthroughCreator {
    flags: PassthroughFlagsFB,
    purpose: PassthroughLayerPurposeFB,
    layer: Option<OxrPassthroughLayer>,
    passthrough: Option<OxrPassthrough>,
}

impl OxrSessionResourceCreator for OxrPassthroughCreator {
    fn initialize(&mut self, world: &mut World) -> Result<()> {
        let session = world.resource::<OxrSession>();

        let passthrough = session.create_passthrough(self.flags)?;

        let layer = session.create_passthrough_layer(&passthrough, self.purpose)?;

        self.layer = Some(layer);
        self.passthrough = Some(passthrough);

        Ok(())
    }

    fn insert_to_world(&mut self, _: &mut World) {}

    fn insert_to_render_world(&mut self, world: &mut World) {
        world.insert_resource(self.passthrough.take().unwrap());
        world.insert_resource(self.layer.take().unwrap());

        world
            .resource_mut::<OxrRenderLayers>()
            .insert(0, Box::new(PassthroughLayer));
    }

    fn remove_from_world(&mut self, _: &mut World) {}

    fn remove_from_render_world(&mut self, world: &mut World) {
        world.remove_resource::<OxrPassthrough>();
        world.remove_resource::<OxrPassthroughLayer>();

        // we don't worry about removing the passthrough render layer here, because it should be done automatically when the session is destroyed
    }
}

pub fn resume_passthrough(
    passthrough: Res<OxrPassthrough>,
    passthrough_layer: Res<OxrPassthroughLayer>,
) {
    passthrough.start().unwrap();
    passthrough_layer.resume().unwrap();
}

pub fn pause_passthrough(
    passthrough: Res<OxrPassthrough>,
    passthrough_layer: Res<OxrPassthroughLayer>,
) {
    passthrough_layer.pause().unwrap();
    passthrough.pause().unwrap();
}

#[inline]
pub fn supports_passthrough(instance: &OxrInstance, system: OxrSystemId) -> Result<bool> {
    if instance.exts().fb_passthrough.is_none() {
        return Ok(false);
    }
    unsafe {
        let mut passthrough = openxr::sys::SystemPassthroughProperties2FB {
            ty: SystemPassthroughProperties2FB::TYPE,
            next: std::ptr::null(),
            capabilities: PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY,
        };
        let mut p = openxr::sys::SystemProperties::out(&mut passthrough as *mut _ as _);
        cvt((instance.fp().get_system_properties)(
            instance.as_raw(),
            system.0,
            p.as_mut_ptr(),
        ))?;
        bevy::log::info!(
            "From supports_passthrough: Passthrough capabilities: {:?}",
            passthrough.capabilities
        );
        Ok(
            (passthrough.capabilities & PassthroughCapabilityFlagsFB::PASSTHROUGH_CAPABILITY)
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
