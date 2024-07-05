//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_mod_openxr::{add_xr_plugins, init::OxrInitPlugin, types::OxrExtensions};
use bevy_mod_xr::session::XrState;
// use openxr::EnvironmentBlendMode;
// use wgpu::TextureFormat;

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).build().set(OxrInitPlugin {
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_hand_tracking();
                exts.other.push("XR_EXTX_overlay\0".into());
                exts
            },
            // blend_modes: Some({
            //     let mut v = Vec::new();
            //     v.push(EnvironmentBlendMode::ALPHA_BLEND);
            //     v.push(EnvironmentBlendMode::ADDITIVE);
            //     v.push(EnvironmentBlendMode::OPAQUE);
            //     v
            // }),
            // formats: Some({
            //     let mut v = Vec::new();
            //     // v.push(TextureFormat::Rgba8Uint);
            //     v.push(TextureFormat::Rgba8Unorm);
            //     v.push(TextureFormat::Rgba8UnormSrgb);
            //     v
            // }),
            ..OxrInitPlugin::default()
        }))
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(bevy_xr_utils::hand_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut end: EventWriter<bevy_mod_xr::session::XrEndSessionEvent>,
    mut destroy: EventWriter<bevy_mod_xr::session::XrDestroySessionEvent>,
    mut begin: EventWriter<bevy_mod_xr::session::XrBeginSessionEvent>,
    mut create: EventWriter<bevy_mod_xr::session::XrCreateSessionEvent>,
    mut request_exit: EventWriter<bevy_mod_xr::session::XrRequestExitEvent>,
    state: Res<XrState>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        info!("sending end");
        end.send_default();
    }
    if keys.just_pressed(KeyCode::KeyC) {
        info!("sending create");
        create.send_default();
    }
    if keys.just_pressed(KeyCode::KeyD) {
        info!("sending destroy");
        destroy.send_default();
    }
    if keys.just_pressed(KeyCode::KeyB) {
        info!("sending begin");
        begin.send_default();
    }
    if keys.just_pressed(KeyCode::KeyR) {
        info!("sending request exit");
        request_exit.send_default();
    }
    if keys.just_pressed(KeyCode::KeyI) {
        info!("current state: {:?}", *state);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Circle::new(4.0)),
    //     material: materials.add(Color::WHITE),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     ..default()
    // });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 2.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
