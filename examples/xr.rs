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

#[derive(Bundle)]
pub struct XrCameraBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub xr_projection: XRProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_3d: Camera3d,
    pub tonemapping: Tonemapping,
    pub dither: DebandDither,
    pub color_grading: ColorGrading,
}

// NOTE: ideally Perspective and Orthographic defaults can share the same impl, but sadly it breaks rust's type inference
impl Default for XrCameraBundle {
    fn default() -> Self {
        Self {
            camera_render_graph: CameraRenderGraph::new(core_3d::graph::NAME),
            camera: Default::default(),
            xr_projection: Default::default(),
            visible_entities: Default::default(),
            frustum: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            camera_3d: Default::default(),
            tonemapping: Default::default(),
            dither: DebandDither::Enabled,
            color_grading: ColorGrading::default(),
        }
    }
}

#[derive(Debug, Clone, Component, Reflect)]
#[reflect(Component, Default)]
pub struct XRProjection {
    pub near: f32,
    pub far: f32,
    #[reflect(ignore)]
    pub fov: Fovf,
}

impl Default for XRProjection {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.,
            fov: Default::default(),
        }
    }
}

impl XRProjection {
    pub fn new(near: f32, far: f32, fov: Fovf) -> Self {
        XRProjection { near, far, fov }
    }
}

impl CameraProjection for XRProjection {
    // =============================================================================
    // math code adapted from
    // https://github.com/KhronosGroup/OpenXR-SDK-Source/blob/master/src/common/xr_linear.h
    // Copyright (c) 2017 The Khronos Group Inc.
    // Copyright (c) 2016 Oculus VR, LLC.
    // SPDX-License-Identifier: Apache-2.0
    // =============================================================================
    fn get_projection_matrix(&self) -> Mat4 {
        //  symmetric perspective for debugging
        // let x_fov = (self.fov.angle_left.abs() + self.fov.angle_right.abs());
        // let y_fov = (self.fov.angle_up.abs() + self.fov.angle_down.abs());
        // return Mat4::perspective_infinite_reverse_rh(y_fov, x_fov / y_fov, self.near);

        let fov = self.fov;
        let is_vulkan_api = false; // FIXME wgpu probably abstracts this
        let near_z = self.near;
        let far_z = -1.; //   use infinite proj
                         // let far_z = self.far;

        let tan_angle_left = fov.angle_left.tan();
        let tan_angle_right = fov.angle_right.tan();

        let tan_angle_down = fov.angle_down.tan();
        let tan_angle_up = fov.angle_up.tan();

        let tan_angle_width = tan_angle_right - tan_angle_left;

        // Set to tanAngleDown - tanAngleUp for a clip space with positive Y
        // down (Vulkan). Set to tanAngleUp - tanAngleDown for a clip space with
        // positive Y up (OpenGL / D3D / Metal).
        // const float tanAngleHeight =
        //     graphicsApi == GRAPHICS_VULKAN ? (tanAngleDown - tanAngleUp) : (tanAngleUp - tanAngleDown);
        let tan_angle_height = if is_vulkan_api {
            tan_angle_down - tan_angle_up
        } else {
            tan_angle_up - tan_angle_down
        };

        // Set to nearZ for a [-1,1] Z clip space (OpenGL / OpenGL ES).
        // Set to zero for a [0,1] Z clip space (Vulkan / D3D / Metal).
        // const float offsetZ =
        //     (graphicsApi == GRAPHICS_OPENGL || graphicsApi == GRAPHICS_OPENGL_ES) ? nearZ : 0;
        // FIXME handle enum of graphics apis
        let offset_z = 0.;

        let mut cols: [f32; 16] = [0.0; 16];

        if far_z <= near_z {
            // place the far plane at infinity
            cols[0] = 2. / tan_angle_width;
            cols[4] = 0.;
            cols[8] = (tan_angle_right + tan_angle_left) / tan_angle_width;
            cols[12] = 0.;

            cols[1] = 0.;
            cols[5] = 2. / tan_angle_height;
            cols[9] = (tan_angle_up + tan_angle_down) / tan_angle_height;
            cols[13] = 0.;

            cols[2] = 0.;
            cols[6] = 0.;
            cols[10] = -1.;
            cols[14] = -(near_z + offset_z);

            cols[3] = 0.;
            cols[7] = 0.;
            cols[11] = -1.;
            cols[15] = 0.;

            //  bevy uses the _reverse_ infinite projection
            //  https://dev.theomader.com/depth-precision/
            let z_reversal = Mat4::from_cols_array_2d(&[
                [1f32, 0., 0., 0.],
                [0., 1., 0., 0.],
                [0., 0., -1., 0.],
                [0., 0., 1., 1.],
            ]);

            return z_reversal * Mat4::from_cols_array(&cols);
        } else {
            // normal projection
            cols[0] = 2. / tan_angle_width;
            cols[4] = 0.;
            cols[8] = (tan_angle_right + tan_angle_left) / tan_angle_width;
            cols[12] = 0.;

            cols[1] = 0.;
            cols[5] = 2. / tan_angle_height;
            cols[9] = (tan_angle_up + tan_angle_down) / tan_angle_height;
            cols[13] = 0.;

            cols[2] = 0.;
            cols[6] = 0.;
            cols[10] = -(far_z + offset_z) / (far_z - near_z);
            cols[14] = -(far_z * (near_z + offset_z)) / (far_z - near_z);

            cols[3] = 0.;
            cols[7] = 0.;
            cols[11] = -1.;
            cols[15] = 0.;
        }

        Mat4::from_cols_array(&cols)
    }

    fn update(&mut self, _width: f32, _height: f32) {}

    fn far(&self) -> f32 {
        self.far
    }
}

use bevy::render::camera::CameraProjectionPlugin;
use bevy::render::view::{update_frusta, VisibilitySystems};
use bevy::transform::TransformSystem;
use bevy::{prelude::*, render::camera::RenderTarget};
use bevy_openxr::input::XrInput;
use bevy_openxr::resources::{XrFrameState, XrSession, XrViews};
use bevy_openxr::xr_input::controllers::XrControllerType;
use bevy_openxr::xr_input::oculus_touch::OculusController;
use bevy_openxr::xr_input::OpenXrInput;
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

#[derive(Component)]
enum CameraType {
    Left,
    Right,
    Middle,
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
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraType::Middle,
    ));
}

fn hands(
    mut gizmos: Gizmos,
    oculus_controller: Res<OculusController>,
    frame_state: Res<XrFrameState>,
    xr_input: Res<XrInput>,
) {
    frame_state.lock().unwrap().map(|a| {
        let right_controller = oculus_controller
            .grip_space
            .right
            .relate(&**&xr_input.stage, a.predicted_display_time)
            .unwrap();
        let left_controller = oculus_controller
            .grip_space
            .left
            .relate(&**&xr_input.stage, a.predicted_display_time)
            .unwrap();
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
    });
}

/*fn hands(
    mut gizmos: Gizmos,
    xr_input: Res<XrInput>,
    session: Res<XrSession>,
    frame_state: Res<XrFrameState>,
) {
    //let pose = xr_input.left_action.create_space(Session::clone(&session), Path, Posef::IDENTITY).unwrap();
    let act = ActiveActionSet::new(&xr_input.action_set);
    session.sync_actions(&[act]).unwrap();
    frame_state.lock().unwrap().map(|a| {
        //let b = pose.locate(&*xr_input.stage, a.predicted_display_time).unwrap();
        let b = xr_input
            .left_space
            .relate(&xr_input.stage, a.predicted_display_time)
            .unwrap();
        gizmos.rect(
            b.0.pose.position.to_vec3(),
            b.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            Color::YELLOW_GREEN,
        );
        let c = xr_input
            .right_space
            .relate(&xr_input.stage, a.predicted_display_time)
            .unwrap();
        gizmos.rect(
            c.0.pose.position.to_vec3(),
            c.0.pose.orientation.to_quat(),
            Vec2::new(0.05, 0.2),
            Color::YELLOW_GREEN,
        )
    });
}*/

fn head_movement(
    views: ResMut<XrViews>,
    mut query: Query<(&mut Transform, &mut Camera, &CameraType, &mut XRProjection)>,
) {
    let views = views.lock().unwrap();
    let mut f = || -> Option<()> {
        let midpoint =
            (views.get(0)?.pose.position.to_vec3() + views.get(1)?.pose.position.to_vec3()) / 2.;
        for (mut t, _, camera_type, _) in query.iter_mut() {
            match camera_type {
                CameraType::Left => t.translation = views.get(0)?.pose.position.to_vec3(),
                CameraType::Right => t.translation = views.get(1)?.pose.position.to_vec3(),
                CameraType::Middle => {
                    t.translation = midpoint;
                }
            }
        }
        let left_rot = views.get(0).unwrap().pose.orientation.to_quat();
        let right_rot = views.get(1).unwrap().pose.orientation.to_quat();
        let mid_rot = if left_rot.dot(right_rot) >= 0. {
            left_rot.slerp(right_rot, 0.5)
        } else {
            right_rot.slerp(left_rot, 0.5)
        };
        for (mut t, _, camera_type, _) in query.iter_mut() {
            match camera_type {
                CameraType::Left => t.rotation = left_rot,
                CameraType::Right => t.rotation = right_rot,
                CameraType::Middle => {
                    t.rotation = mid_rot;
                }
            }
        }

        for (mut transform, _cam, camera_type, mut xr_projection) in query.iter_mut() {
            let view_idx = match camera_type {
                CameraType::Left => 0,
                CameraType::Right => 1,
                CameraType::Middle => panic!(),
            };
            let view = views.get(view_idx).unwrap();
            xr_projection.fov = view.fov;

            transform.rotation = view.pose.orientation.to_quat();
            let pos = view.pose.position;
            transform.translation = pos.to_vec3();
        }

        Some(())
    };
    f();
}
pub trait Vec3Conv {
    fn to_vec3(&self) -> Vec3;
}

impl Vec3Conv for openxr::Vector3f {
    fn to_vec3(&self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}
pub trait QuatConv {
    fn to_quat(&self) -> Quat;
}

impl QuatConv for openxr::Quaternionf {
    fn to_quat(&self) -> Quat {
        Quat::from_xyzw(self.x, self.y, self.z, self.w)
    }
}
