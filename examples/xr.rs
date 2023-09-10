use bevy::core_pipeline::core_3d;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::ecs::prelude::{Bundle, Component, ReflectComponent};

use bevy::math::Mat4;
use bevy::prelude::Camera3d;
use bevy::reflect::{std_traits::ReflectDefault, Reflect};
use bevy::render::view::ColorGrading;
use bevy::render::{
    camera::{Camera, CameraProjection, CameraRenderGraph},
    primitives::Frustum,
    view::VisibleEntities,
};
use bevy::transform::components::{GlobalTransform, Transform};
//  mostly copied from https://github.com/blaind/bevy_openxr/tree/main/crates/bevy_openxr/src/render_graph/camera
use openxr::Fovf;

use bevy::render::camera::CameraProjectionPlugin;
use bevy::render::view::{update_frusta, VisibilitySystems};
use bevy::transform::TransformSystem;
use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrSession, XrViews};
use bevy_openxr::xr_input::controllers::XrControllerType;
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::{OpenXrInput, QuatConv, Vec3Conv};
use bevy_openxr::{DefaultXrPlugins, LEFT_XR_TEXTURE_HANDLE, RIGHT_XR_TEXTURE_HANDLE};
use openxr::ActiveActionSet;

fn main() {
    color_eyre::install().unwrap();

    info!("Running `openxr-6dof` skill");
    App::new()
        .add_plugins(DefaultXrPlugins)
        .add_plugins(OpenXrInput::new(XrControllerType::OculusTouch))
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
) {
    let mut func = || -> anyhow::Result<()> {
        let frame_state = *frame_state.lock().unwrap();

        let right_controller = oculus_controller
            .grip_space
            .right
            .relate(&**&xr_input.stage, frame_state.predicted_display_time)?;
        let left_controller = oculus_controller
            .grip_space
            .left
            .relate(&**&xr_input.stage, frame_state.predicted_display_time)?;
        gizmos.rect(
            right_controller.0.pose.position.to_vec3(),
            right_controller.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            Color::YELLOW_GREEN,
        );
        gizmos.rect(
            left_controller.0.pose.position.to_vec3(),
            left_controller.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            Color::YELLOW_GREEN,
        );
        Ok(())
    };

    let _ = func();
}
