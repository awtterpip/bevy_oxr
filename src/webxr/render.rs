use crate::render::{XrView, XrViews};
use crate::types::Pose;

use super::resources::*;

use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::ecs::system::{NonSend, Res, ResMut};
use bevy::ecs::world::World;
use bevy::math::{quat, uvec2, vec3, Mat4};
use bevy::render::camera::{
    ManualTextureView, ManualTextureViewHandle, ManualTextureViews, RenderTarget, Viewport,
};
use bevy::render::renderer::RenderDevice;
use bevy::utils::default;

pub const XR_TEXTURE_VIEW_HANDLE: ManualTextureViewHandle =
    ManualTextureViewHandle(crate::render::XR_TEXTURE_VIEW_INDEX);

pub struct XrRenderingPlugin;

impl Plugin for XrRenderingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<XrViews>();
        app.add_systems(
            PreUpdate,
            (insert_gl_layer, update_manual_texture_views, insert_views).chain(),
        );
    }
}

pub fn insert_gl_layer(world: &mut World) {
    let gl_layer = world
        .non_send_resource::<XrFrame>()
        .session()
        .render_state()
        .base_layer()
        .unwrap();
    world.insert_non_send_resource(XrWebGlLayer(gl_layer));
}

pub fn update_manual_texture_views(
    gl_layer: NonSend<XrWebGlLayer>,
    render_device: Res<RenderDevice>,
    mut manual_tex_view: ResMut<ManualTextureViews>,
) {
    let dest_texture = create_framebuffer_texture(render_device.wgpu_device(), &gl_layer);
    let view = dest_texture.create_view(&default());

    manual_tex_view.insert(
        XR_TEXTURE_VIEW_HANDLE,
        ManualTextureView::with_default_format(
            view.into(),
            uvec2(gl_layer.framebuffer_width(), gl_layer.framebuffer_height()),
        ),
    );
}

pub fn insert_views(
    gl_layer: NonSend<XrWebGlLayer>,
    reference_space: NonSend<XrReferenceSpace>,
    frame: NonSend<XrFrame>,
    mut xr_views: ResMut<XrViews>,
) {
    let Some(viewer_pose) = frame.get_viewer_pose(&reference_space) else {
        return;
    };

    let views = viewer_pose
        .views()
        .into_iter()
        .map(Into::<web_sys::XrView>::into)
        .map(|view| {
            let transform = view.transform();
            let position = transform.position();
            let orientation = transform.orientation();
            let viewport = gl_layer
                .get_viewport(&view)
                .map(|viewport| Viewport {
                    physical_position: uvec2(viewport.x() as u32, viewport.y() as u32),
                    physical_size: uvec2(viewport.width() as u32, viewport.height() as u32),
                    ..Default::default()
                })
                .unwrap();
            XrView {
                projection_matrix: Mat4::from_cols_array(
                    &view.projection_matrix().try_into().unwrap(),
                ),
                pose: Pose {
                    translation: vec3(
                        position.x() as f32,
                        position.y() as f32,
                        position.z() as f32,
                    ),
                    rotation: quat(
                        orientation.x() as f32,
                        orientation.y() as f32,
                        orientation.z() as f32,
                        orientation.w() as f32,
                    ),
                },
                render_target: RenderTarget::TextureView(XR_TEXTURE_VIEW_HANDLE),
                view_port: Some(viewport),
            }
        })
        .collect();
    xr_views.0 = views;
}

pub fn create_framebuffer_texture(device: &wgpu::Device, gl_layer: &XrWebGlLayer) -> wgpu::Texture {
    unsafe {
        device.create_texture_from_hal::<wgpu_hal::gles::Api>(
            wgpu_hal::gles::Texture {
                inner: wgpu_hal::gles::TextureInner::ExternalFramebuffer {
                    // inner: framebuffer,
                    inner: gl_layer.framebuffer_unwrapped(),
                    // inner: framebuffer.as_ref().unwrap().clone(),
                },
                mip_level_count: 1,
                array_layer_count: 1,
                format: wgpu::TextureFormat::Rgba8Unorm, //TODO check this is ok, different from bevy default
                format_desc: wgpu_hal::gles::TextureFormatDesc {
                    internal: glow::RGBA,
                    external: glow::RGBA,
                    data_type: glow::UNSIGNED_BYTE,
                },
                copy_size: wgpu_hal::CopyExtent {
                    width: gl_layer.framebuffer_width(),
                    height: gl_layer.framebuffer_height(),
                    depth: 1,
                },
                drop_guard: None,
                is_cubemap: false,
            },
            &wgpu::TextureDescriptor {
                label: Some("framebuffer (color)"),
                size: wgpu::Extent3d {
                    width: gl_layer.framebuffer_width(),
                    height: gl_layer.framebuffer_height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING,
                // | wgpu::TextureUsages::COPY_SRC,
                // | wgpu::TextureUsages::COPY_DST,
            },
        )
    }
}
