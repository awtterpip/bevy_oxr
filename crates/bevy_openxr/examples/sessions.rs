//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;
use bevy_mod_openxr::add_xr_plugins;
use bevy_mod_xr::session::{XrSessionPlugin, XrState};

fn main() -> AppExit {
    App::new()
        .add_plugins(add_xr_plugins(DefaultPlugins).set(XrSessionPlugin { auto_handle: true }))
        .add_plugins(bevy_mod_xr::hand_debug_gizmos::HandGizmosPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .insert_resource(AmbientLight::default())
        .run()
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
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
