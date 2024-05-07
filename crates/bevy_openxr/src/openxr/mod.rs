// use actions::XrActionPlugin;
use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    render::{pipelined_rendering::PipelinedRenderingPlugin, RenderPlugin},
    utils::default,
    window::{PresentMode, Window, WindowPlugin},
};
use bevy_xr::camera::XrCameraPlugin;
use bevy_xr::session::XrSessionPlugin;
use init::OxrInitPlugin;
use render::OxrRenderPlugin;

use self::{exts::OxrExtensions, features::passthrough::OxrPassthroughPlugin};

pub mod error;
mod exts;
pub mod features;
pub mod graphics;
pub mod init;
pub mod layer_builder;
pub mod render;
pub mod resources;
pub mod types;
pub mod action_binding;
pub mod action_set_attaching;
pub mod reference_space;
pub mod helper_traits;

pub fn add_xr_plugins<G: PluginGroup>(plugins: G) -> PluginGroupBuilder {
    plugins
        .build()
        .disable::<RenderPlugin>()
        .disable::<PipelinedRenderingPlugin>()
        .add_before::<RenderPlugin, _>(XrSessionPlugin)
        .add_before::<RenderPlugin, _>(OxrInitPlugin {
            app_info: default(),
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_fb_passthrough();
                exts
            },
            blend_modes: default(),
            backends: default(),
            formats: Some(vec![wgpu::TextureFormat::Rgba8UnormSrgb]),
            resolutions: default(),
            synchronous_pipeline_compilation: default(),
        })
        .add(OxrRenderPlugin)
        .add(OxrPassthroughPlugin)
        .add(XrCameraPlugin)
        .add(action_set_attaching::OxrActionAttachingPlugin)
        .add(action_binding::OxrActionBindingPlugin)
        // .add(XrActionPlugin)
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
