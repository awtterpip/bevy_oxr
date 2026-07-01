//! Requests OpenXR stereo multiview resources.
//!
//! Bevy's built-in 3D render graph still renders through the per-eye texture views. Custom render
//! graph nodes can target `xr_multiview_texture_view_handle()` and set the render pass
//! `multiview_mask` to the configured `XrStereoRenderMode::view_mask()`.

use bevy::{prelude::*, render::pipelined_rendering::PipelinedRenderingPlugin};
use bevy_mod_openxr::{
    add_xr_plugins,
    render::{xr_multiview_texture_view_handle, XR_MULTIVIEW_TEXTURE_INDEX},
};
use bevy_mod_xr::camera::XrStereoRenderMode;

fn main() -> AppExit {
    App::new()
        .insert_resource(XrStereoRenderMode::stereo_multiview())
        .add_plugins(add_xr_plugins(
            DefaultPlugins.build().disable::<PipelinedRenderingPlugin>(),
        ))
        .add_systems(Startup, (log_multiview_target, setup))
        .run()
}

fn log_multiview_target(stereo_render_mode: Res<XrStereoRenderMode>) {
    let _handle = xr_multiview_texture_view_handle();
    info!(
        "OpenXR stereo multiview target requested at manual texture view index {}; pass mask: {:?}",
        XR_MULTIVIEW_TEXTURE_INDEX,
        stereo_render_mode.view_mask()
    );
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    commands.spawn((
        PointLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
}
