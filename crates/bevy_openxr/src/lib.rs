use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    render::RenderPlugin,
    utils::default,
};
use bevy_xr::camera::XrCameraPlugin;
use init::XrInitPlugin;
use render::XrRenderPlugin;

pub mod camera;
pub mod error;
pub mod extensions;
pub mod graphics;
pub mod init;
pub mod layer_builder;
pub mod render;
pub mod resources;
pub mod types;

pub fn add_xr_plugins<G: PluginGroup>(plugins: G) -> PluginGroupBuilder {
    plugins
        .build()
        .disable::<RenderPlugin>()
        .add_before::<RenderPlugin, _>(bevy_xr::session::XrSessionPlugin)
        .add_before::<RenderPlugin, _>(XrInitPlugin {
            app_info: default(),
            exts: default(),
            blend_modes: default(),
            backends: default(),
            formats: Some(vec![wgpu::TextureFormat::Rgba8UnormSrgb]),
            resolutions: default(),
            synchronous_pipeline_compilation: default(),
        })
        .add(XrRenderPlugin)
        .add(XrCameraPlugin)
}
