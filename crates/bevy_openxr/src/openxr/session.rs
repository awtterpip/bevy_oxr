use crate::init::OxrPreUpdateSet;
use crate::resources::{
    OxrCleanupSession, OxrPassthrough, OxrPassthroughLayer, OxrSessionStarted, OxrSwapchain,
};
use crate::types::{Result, SwapchainCreateInfo};
use bevy::ecs::event::ManualEventReader;
use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy_xr::session::{
    status_changed_to, XrSessionCreated, XrSessionExiting, XrSharedStatus, XrStatus,
};
use openxr::AnyGraphics;

use crate::graphics::{graphics_match, GraphicsExt, GraphicsType, GraphicsWrap};

#[derive(Event, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OxrSessionStatusEvent {
    Created,
    AboutToBeDestroyed,
}

pub struct OxrSessionPlugin;

impl Plugin for OxrSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<OxrSessionStatusEvent>();
        app.add_systems(
            PreUpdate,
            run_session_status_schedules.in_set(OxrPreUpdateSet::HandleEvents),
        );
        app.add_systems(XrSessionExiting, clean_session);
        app.sub_app_mut(RenderApp)
            .add_systems(XrSessionExiting, |mut cmds: Commands| {
                cmds.remove_resource::<OxrCleanupSession>();
            });
        app.add_systems(
            PreUpdate,
            handle_stopping_state.run_if(status_changed_to(XrStatus::Stopping)),
        );
    }
}

fn handle_stopping_state(session: Res<OxrSession>, session_started: Res<OxrSessionStarted>) {
    session.end().expect("Failed to end session");
    session_started.set(false);
}

fn clean_session(world: &mut World) {
    world.insert_resource(OxrCleanupSession(true));
    // It should be impossible to call this if the session is Unavailable
    world
        .get_resource::<XrSharedStatus>()
        .unwrap()
        .set(XrStatus::Available);
}

#[derive(Resource, Default)]
struct SessionStatusReader(ManualEventReader<OxrSessionStatusEvent>);

fn run_session_status_schedules(world: &mut World) {
    let mut reader = world
        .remove_resource::<SessionStatusReader>()
        .unwrap_or_default();

    let events = reader
        .0
        .read(
            world
                .get_resource::<Events<OxrSessionStatusEvent>>()
                .unwrap(),
        )
        .copied()
        .collect::<Vec<_>>();
    for e in events.iter() {
        match e {
            OxrSessionStatusEvent::Created => {
                world.run_schedule(XrSessionCreated);
                world.run_system_once(apply_deferred);
            }
            OxrSessionStatusEvent::AboutToBeDestroyed => {
                world.run_schedule(XrSessionExiting);
                world.run_system_once(apply_deferred);
                world.remove_resource::<OxrSession>();
            }
        }
    }
    world.insert_resource(reader);
}

/// Graphics agnostic wrapper around [openxr::Session].
///
/// See [`openxr::Session`] for other available methods.
#[derive(Resource, Deref, Clone)]
pub struct OxrSession(
    /// A session handle with [`AnyGraphics`].
    /// Having this here allows the majority of [`Session`](openxr::Session)'s methods to work without having to rewrite them.
    #[deref]
    pub(crate) openxr::Session<AnyGraphics>,
    /// A [`GraphicsWrap`] with [`openxr::Session<G>`] as the inner type.
    /// This is so that we can still operate on functions that don't take [`AnyGraphics`] as the generic.
    pub(crate) GraphicsWrap<Self>,
);

impl GraphicsType for OxrSession {
    type Inner<G: GraphicsExt> = openxr::Session<G>;
}

impl<G: GraphicsExt> From<openxr::Session<G>> for OxrSession {
    fn from(session: openxr::Session<G>) -> Self {
        Self::from_inner(session)
    }
}

impl OxrSession {
    /// Creates a new [`OxrSession`] from an [`openxr::Session`].
    /// In the majority of cases, you should use [`create_session`](OxrInstance::create_session) instead.
    pub fn from_inner<G: GraphicsExt>(session: openxr::Session<G>) -> Self {
        Self(session.clone().into_any_graphics(), G::wrap(session))
    }

    /// Returns [`GraphicsWrap`] with [`openxr::Session<G>`] as the inner type.
    ///
    /// This can be useful if you need access to the original [`openxr::Session`] with the graphics API still specified.
    pub fn typed_session(&self) -> &GraphicsWrap<Self> {
        &self.1
    }

    /// Enumerates all available swapchain formats and converts them to wgpu's [`TextureFormat`](wgpu::TextureFormat).
    ///
    /// Calls [`enumerate_swapchain_formats`](openxr::Session::enumerate_swapchain_formats) internally.
    pub fn enumerate_swapchain_formats(&self) -> Result<Vec<wgpu::TextureFormat>> {
        graphics_match!(
            &self.1;
            session => Ok(session.enumerate_swapchain_formats()?.into_iter().filter_map(Api::into_wgpu_format).collect())
        )
    }

    /// Creates an [OxrSwapchain].
    ///
    /// Calls [`create_swapchain`](openxr::Session::create_swapchain) internally.
    pub fn create_swapchain(&self, info: SwapchainCreateInfo) -> Result<OxrSwapchain> {
        Ok(OxrSwapchain(graphics_match!(
            &self.1;
            session => session.create_swapchain(&info.try_into()?)? => OxrSwapchain
        )))
    }

    /// Creates a passthrough.
    ///
    /// Requires [`XR_FB_passthrough`](https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_FB_passthrough).
    ///
    /// Calls [`create_passthrough`](openxr::Session::create_passthrough) internally.
    pub fn create_passthrough(&self, flags: openxr::PassthroughFlagsFB) -> Result<OxrPassthrough> {
        Ok(OxrPassthrough(
            graphics_match! {
                &self.1;
                session => session.create_passthrough(flags)?
            },
            flags,
        ))
    }

    /// Creates a passthrough layer that can be used to make a [`CompositionLayerPassthrough`](crate::layer_builder::CompositionLayerPassthrough) for frame submission.
    ///
    /// Requires [`XR_FB_passthrough`](https://www.khronos.org/registry/OpenXR/specs/1.0/html/xrspec.html#XR_FB_passthrough).
    ///
    /// Calls [`create_passthrough_layer`](openxr::Session::create_passthrough_layer) internally.
    pub fn create_passthrough_layer(
        &self,
        passthrough: &OxrPassthrough,
        purpose: openxr::PassthroughLayerPurposeFB,
    ) -> Result<OxrPassthroughLayer> {
        Ok(OxrPassthroughLayer(graphics_match! {
            &self.1;
            session => session.create_passthrough_layer(&passthrough.0, passthrough.1, purpose)?
        }))
    }
}
