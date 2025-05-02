//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_mod_openxr::{
    add_xr_plugins, features::overlay::OxrOverlaySessionEvent, init::OxrInitPlugin,
    resources::OxrSessionConfig, types::OxrExtensions,
};
use openxr::EnvironmentBlendMode;

fn main() {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).build().set(OxrInitPlugin {
            exts: {
                let mut exts = OxrExtensions::default();
                exts.enable_hand_tracking();
                exts.extx_overlay = true;
                exts
            },
            ..OxrInitPlugin::default()
        }))
        .insert_resource(OxrSessionConfig {
            blend_modes: Some({
                vec![
                    EnvironmentBlendMode::ALPHA_BLEND,
                    EnvironmentBlendMode::OPAQUE,
                ]
            }),
            ..OxrSessionConfig::default()
        })
        .insert_resource(ClearColor(Color::NONE))
        .add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .add_systems(Update, print_main_session_changes)
        .run();
}

fn print_main_session_changes(mut events: EventReader<OxrOverlaySessionEvent>) {
    for event in events.read() {
        let OxrOverlaySessionEvent::MainSessionVisibilityChanged { visible, flags: _ } = event;
        info!("main session visible: {visible}");
    }
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut end: EventWriter<bevy_mod_xr::session::XrEndSessionEvent>,
    mut destroy: EventWriter<bevy_mod_xr::session::XrDestroySessionEvent>,
    mut begin: EventWriter<bevy_mod_xr::session::XrBeginSessionEvent>,
    mut create: EventWriter<bevy_mod_xr::session::XrCreateSessionEvent>,
    mut request_exit: EventWriter<bevy_mod_xr::session::XrRequestExitEvent>,
) {
    if keys.just_pressed(KeyCode::KeyE) {
        info!("sending end");
        end.write_default();
    }
    if keys.just_pressed(KeyCode::KeyC) {
        info!("sending create");
        create.write_default();
    }
    if keys.just_pressed(KeyCode::KeyD) {
        info!("sending destroy");
        destroy.write_default();
    }
    if keys.just_pressed(KeyCode::KeyB) {
        info!("sending begin");
        begin.write_default();
    }
    if keys.just_pressed(KeyCode::KeyR) {
        info!("sending request exit");
        request_exit.write_default();
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 2.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
