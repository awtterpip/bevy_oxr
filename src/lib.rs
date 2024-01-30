use std::sync::Arc;

use bevy::{
    app::PluginGroupBuilder,
    prelude::*,
    render::{
        camera::{ManualTextureView, ManualTextureViewHandle, ManualTextureViews},
        pipelined_rendering::PipelinedRenderingPlugin,
        renderer::{render_system, RenderAdapter, RenderAdapterInfo, RenderInstance, RenderQueue},
        Render, RenderApp, RenderPlugin,
    },
    window::PresentMode,
};
use xr_api::prelude::*;

pub const LEFT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(1208214591);
pub const RIGHT_XR_TEXTURE_HANDLE: ManualTextureViewHandle = ManualTextureViewHandle(3383858418);

pub struct XrPlugin;

impl Plugin for XrPlugin {
    fn build(&self, app: &mut App) {
        let instance = Entry::new()
            .create_instance(ExtensionSet { vulkan: true })
            .unwrap();
        let session = instance
            .create_session(SessionCreateInfo {
                texture_format: wgpu::TextureFormat::Rgba8UnormSrgb,
            })
            .unwrap();

        let (device, queue, adapter_info, adapter, instance) =
            session.get_render_resources().unwrap();

        app.insert_non_send_resource(session.clone());
        app.add_plugins(RenderPlugin {
            render_creation: bevy::render::settings::RenderCreation::Manual(
                device.into(),
                RenderQueue(Arc::new(queue)),
                RenderAdapterInfo(adapter_info),
                RenderAdapter(Arc::new(adapter)),
                RenderInstance(Arc::new(instance)),
            ),
        });

        app.add_systems(Last, begin_frame);
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_non_send_resource(session);
        render_app.add_systems(Render, end_frame.after(render_system));
    }
}

pub fn begin_frame(
    session: NonSend<Session>,
    mut manual_texture_views: ResMut<ManualTextureViews>,
) {
    let (left, right) = session.begin_frame().unwrap();

    let left = ManualTextureView {
        texture_view: left.texture_view().unwrap().into(),
        size: left.resolution(),
        format: left.format(),
    };
    let right = ManualTextureView {
        texture_view: right.texture_view().unwrap().into(),
        size: right.resolution(),
        format: right.format(),
    };

    manual_texture_views.insert(LEFT_XR_TEXTURE_HANDLE, left);
    manual_texture_views.insert(RIGHT_XR_TEXTURE_HANDLE, right);
}

pub fn end_frame(session: NonSend<Session>) {
    session.end_frame().unwrap();
}

pub struct DefaultXrPlugins;

impl PluginGroup for DefaultXrPlugins {
    fn build(self) -> PluginGroupBuilder {
        DefaultPlugins
            .build()
            .disable::<RenderPlugin>()
            .disable::<PipelinedRenderingPlugin>()
            .add_before::<RenderPlugin, _>(XrPlugin)
            .set(WindowPlugin {
                #[cfg(not(target_os = "android"))]
                primary_window: Some(Window {
                    transparent: true,
                    present_mode: PresentMode::AutoNoVsync,
                    ..default()
                }),
                #[cfg(target_os = "android")]
                primary_window: None,
                #[cfg(target_os = "android")]
                exit_condition: bevy::window::ExitCondition::DontExit,
                #[cfg(target_os = "android")]
                close_when_requested: true,
                ..default()
            })
    }
}
