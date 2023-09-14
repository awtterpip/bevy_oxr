use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrInstance, XrSession};
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::{Hand, QuatConv, Vec3Conv};
use bevy_openxr::DefaultXrPlugins;

fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, hands)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    },));
}

fn hands(
    mut gizmos: Gizmos,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
    instance: Res<XrInstance>,
    session: Res<XrSession>,
) {
    let mut func = || -> color_eyre::Result<()> {
        let frame_state = *frame_state.lock().unwrap();

        let controller = oculus_controller.get_ref(&instance, &session, &frame_state, &xr_input);

        let right_controller = controller.grip_space(Hand::Right);
        let left_controller = controller.grip_space(Hand::Left);

        let mut color = Color::YELLOW_GREEN;
        if controller.a_button() {
            color = Color::BLUE;
        }
        if controller.b_button() {
            color = Color::RED;
        }
        if controller.trigger(Hand::Right) != 0.0 {
            color = Color::rgb(
                controller.trigger(Hand::Right),
                0.5,
                controller.trigger(Hand::Right),
            );
        }

        gizmos.rect(
            right_controller.0.pose.position.to_vec3(),
            right_controller.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            color,
        );
        gizmos.rect(
            left_controller.0.pose.position.to_vec3(),
            left_controller.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            color,
        );
        Ok(())
    };

    let _ = func();
}
