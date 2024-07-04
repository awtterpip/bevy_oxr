use bevy::color::palettes;
use bevy::render::extract_resource::ExtractResource;
use bevy::{prelude::*, render::extract_resource::ExtractResourcePlugin};
use std::{marker::PhantomData, mem, ptr};

use crate::resources::XrSession;
use crate::{
    resources::XrInstance,
    xr_arc_resource_wrapper,
    xr_init::{XrCleanup, XrSetup},
};
use openxr as xr;
use xr::{
    sys::{Space, SystemPassthroughProperties2FB},
    CompositionLayerBase, CompositionLayerFlags, FormFactor, Graphics,
    PassthroughCapabilityFlagsFB,
};

#[derive(
    Clone, Copy, Default, Debug, Resource, PartialEq, PartialOrd, Ord, Eq, Reflect, ExtractResource,
)]
pub enum XrPassthroughState {
    #[default]
    Unsupported,
    Running,
    Paused,
}

xr_arc_resource_wrapper!(XrPassthrough, xr::Passthrough);
xr_arc_resource_wrapper!(XrPassthroughLayer, xr::PassthroughLayer);

pub struct PassthroughPlugin;

impl Plugin for PassthroughPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ResumePassthrough>();
        app.add_event::<PausePassthrough>();
        app.add_plugins(ExtractResourcePlugin::<XrPassthroughLayer>::default());
        app.add_plugins(ExtractResourcePlugin::<XrPassthroughState>::default());
        app.register_type::<XrPassthroughState>();
        app.add_systems(Startup, check_passthrough_support);
        app.add_systems(
            XrSetup,
            setup_passthrough
                .run_if(|state: Res<XrPassthroughState>| *state != XrPassthroughState::Unsupported),
        );
        app.add_systems(XrCleanup, cleanup_passthrough);
        app.add_systems(
            Update,
            resume_passthrough.run_if(
                resource_exists_and_equals(XrPassthroughState::Paused)
                    .and_then(on_event::<ResumePassthrough>()),
            ),
        );
        app.add_systems(
            Update,
            pause_passthrough.run_if(
                resource_exists_and_equals(XrPassthroughState::Running)
                    .and_then(on_event::<PausePassthrough>()),
            ),
        );
    }
}

fn check_passthrough_support(mut cmds: Commands, instance: Option<Res<XrInstance>>) {
    match instance {
        None => cmds.insert_resource(XrPassthroughState::Unsupported),
        Some(instance) => {
            let supported = instance.exts().fb_passthrough.is_some()
                && supports_passthrough(
                    &instance,
                    instance.system(FormFactor::HEAD_MOUNTED_DISPLAY).unwrap(),
                )
                .is_ok_and(|v| v);
            match supported {
                false => cmds.insert_resource(XrPassthroughState::Unsupported),
                true => cmds.insert_resource(XrPassthroughState::Paused),
            }
        }
    }
}

fn resume_passthrough(
    layer: Res<XrPassthroughLayer>,
    mut state: ResMut<XrPassthroughState>,
    mut clear_color: ResMut<ClearColor>,
) {
    if let Err(e) = layer.resume() {
        warn!("Unable to resume Passthrough: {}", e);
        return;
    }
    **clear_color = Srgba::NONE.into();
    *state = XrPassthroughState::Running;
}
fn pause_passthrough(
    layer: Res<XrPassthroughLayer>,
    mut state: ResMut<XrPassthroughState>,
    mut clear_color: ResMut<ClearColor>,
) {
    if let Err(e) = layer.pause() {
        warn!("Unable to resume Passthrough: {}", e);
        return;
    }
    clear_color.set_alpha(1.0);
    *state = XrPassthroughState::Paused;
}

fn cleanup_passthrough(mut cmds: Commands) {
    cmds.remove_resource::<XrPassthrough>();
    cmds.remove_resource::<XrPassthroughLayer>();
}

fn setup_passthrough(mut cmds: Commands, session: Res<XrSession>) {
    match create_passthrough(&session) {
        Ok((passthrough, layer)) => {
            cmds.insert_resource(XrPassthrough::from(passthrough));
            cmds.insert_resource(XrPassthroughLayer::from(layer));
        }
        Err(e) => {
            warn!("Unable to create passthrough: {}", e);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Reflect, Event)]
pub struct ResumePassthrough;

#[derive(Clone, Copy, Debug, Default, Reflect, Event)]
pub struct PausePassthrough;

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
    pub(crate) fn from_xr_passthrough_layer(layer: &XrPassthroughLayer) -> Self {
        Self {
            inner: xr::sys::CompositionLayerPassthroughFB {
                ty: xr::sys::CompositionLayerPassthroughFB::TYPE,
                next: ptr::null(),
                flags: CompositionLayerFlags::BLEND_TEXTURE_SOURCE_ALPHA,
                space: Space::NULL,
                layer_handle: *layer.inner(),
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

#[inline]
pub fn create_passthrough(
    xr_session: &XrSession,
) -> xr::Result<(xr::Passthrough, xr::PassthroughLayer)> {
    let flags = xr::PassthroughFlagsFB::IS_RUNNING_AT_CREATION;
    let purpose = xr::PassthroughLayerPurposeFB::RECONSTRUCTION;
    let passthrough = match xr_session {
        #[cfg(feature = "vulkan")]
        XrSession::Vulkan(session) => {
            session.create_passthrough(xr::PassthroughFlagsFB::IS_RUNNING_AT_CREATION)
        }
        #[cfg(all(feature = "d3d12", windows))]
        XrSession::D3D12(session) => {
            session.create_passthrough(xr::PassthroughFlagsFB::IS_RUNNING_AT_CREATION)
        }
    }?;
    let passthrough_layer = match xr_session {
        #[cfg(feature = "vulkan")]
        XrSession::Vulkan(session) => {
            session.create_passthrough_layer(&passthrough, flags, purpose)
        }
        #[cfg(all(feature = "d3d12", windows))]
        XrSession::D3D12(session) => session.create_passthrough_layer(&passthrough, flags, purpose),
    }?;
    Ok((passthrough, passthrough_layer))
}

/// Enable Passthrough on xr startup
/// just sends the [`ResumePassthrough`] event in [`XrSetup`]
pub struct EnablePassthroughStartup;

impl Plugin for EnablePassthroughStartup {
    fn build(&self, app: &mut App) {
        app.add_systems(XrSetup, |mut e: EventWriter<ResumePassthrough>| {
            e.send_default();
        });
    }
}
