// use actions::XrActionPlugin;
use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    render::RenderPlugin,
    utils::default,
    window::{PresentMode, Window, WindowPlugin},
};
use bevy_xr::camera::XrCameraPlugin;
use bevy_xr::session::XrSessionPlugin;
use init::OxrInitPlugin;
use render::OxrRenderPlugin;

use self::{
    features::{handtracking::HandTrackingPlugin, passthrough::OxrPassthroughPlugin},
    reference_space::OxrReferenceSpacePlugin,
};

pub mod action_binding;
pub mod action_set_attaching;
pub mod action_set_syncing;
pub mod error;
mod exts;
pub mod features;
pub mod graphics;
pub mod helper_traits;
pub mod init;
pub mod layer_builder;
pub mod next_chain;
pub mod reference_space;
pub mod render;
pub mod resources;
pub mod session;
pub mod spaces;
pub mod types;

pub fn add_xr_plugins<G: PluginGroup>(plugins: G) -> PluginGroupBuilder {
    plugins
        .build()
        .disable::<RenderPlugin>()
        // .disable::<PipelinedRenderingPlugin>()
        .add_before::<RenderPlugin, _>(XrSessionPlugin { auto_handle: true })
        .add_before::<RenderPlugin, _>(OxrInitPlugin::default())
        .add(OxrReferenceSpacePlugin::default())
        .add(OxrRenderPlugin)
        .add(OxrPassthroughPlugin)
        .add(HandTrackingPlugin::default())
        .add(XrCameraPlugin)
        .add(action_set_attaching::OxrActionAttachingPlugin)
        .add(action_binding::OxrActionBindingPlugin)
        .add(action_set_syncing::OxrActionSyncingPlugin)
        .add(features::overlay::OxrOverlayPlugin)
        .add(spaces::OxrSpatialPlugin)
        .add(spaces::OxrSpacePatchingPlugin)
        // .add(XrActionPlugin)
        // we should probably handle the exiting ourselfs so that we can correctly end the
        // session and instance
        .set(WindowPlugin {
            primary_window: Some(Window {
                transparent: true,
                present_mode: PresentMode::AutoNoVsync,
                // title: self.app_info.name.clone(),
                ..default()
            }),
            // #[cfg(target_os = "android")]
            // exit_condition: bevy::window::ExitCondition::DontExit,
            #[cfg(target_os = "android")]
            close_when_requested: true,
            ..default()
        })
}
