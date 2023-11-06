use crate::xr_input::{QuatConv, Vec3Conv};
use crate::{LEFT_XR_TEXTURE_HANDLE, RIGHT_XR_TEXTURE_HANDLE};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, CameraRenderGraph, RenderTarget};
use bevy::render::primitives::Frustum;
use bevy::render::view::{ColorGrading, VisibleEntities};
use openxr::Fovf;

#[derive(Bundle)]
pub struct XrCamerasBundle {
    pub left: XrCameraBundle,
    pub right: XrCameraBundle,
}
impl XrCamerasBundle {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Default for XrCamerasBundle {
    fn default() -> Self {
        Self {
            left: XrCameraBundle::new(Eye::Left),
            right: XrCameraBundle::new(Eye::Right),
        }
    }
}

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
    pub xr_camera_type: XrCameraType,
}
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Component)]
pub enum XrCameraType {
    Xr(Eye),
    Flatscreen,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum Eye {
    Left = 0,
    Right = 1,
}

impl XrCameraBundle {
    pub fn new(eye: Eye) -> Self {
        Self {
            camera: Camera {
                order: -1,
                target: RenderTarget::TextureView(match eye {
                    Eye::Left => LEFT_XR_TEXTURE_HANDLE,
                    Eye::Right => RIGHT_XR_TEXTURE_HANDLE,
                }),
                viewport: None,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(bevy::core_pipeline::core_3d::graph::NAME),
            xr_projection: Default::default(),
            visible_entities: Default::default(),
            frustum: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            camera_3d: Default::default(),
            tonemapping: Default::default(),
            dither: DebandDither::Enabled,
            color_grading: Default::default(),
            xr_camera_type: XrCameraType::Xr(eye),
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

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        let tan_angle_left = self.fov.angle_left.tan();
        let tan_angle_right = self.fov.angle_right.tan();

        let tan_angle_bottom = self.fov.angle_down.tan();
        let tan_angle_top = self.fov.angle_up.tan();

        // NOTE: These vertices are in the specific order required by [`calculate_cascade`].
        [
            Vec3A::new(tan_angle_right, tan_angle_bottom, 1.0) * z_near, // bottom right
            Vec3A::new(tan_angle_right, tan_angle_top, 1.0) * z_near,    // top right
            Vec3A::new(tan_angle_left, tan_angle_top, 1.0) * z_near,     // top left
            Vec3A::new(tan_angle_left, tan_angle_bottom, 1.0) * z_near,  // bottom left
            Vec3A::new(tan_angle_right, tan_angle_bottom, 1.0) * z_far,  // bottom right
            Vec3A::new(tan_angle_right, tan_angle_top, 1.0) * z_far,     // top right
            Vec3A::new(tan_angle_left, tan_angle_top, 1.0) * z_far,      // top left
            Vec3A::new(tan_angle_left, tan_angle_bottom, 1.0) * z_far,   // bottom left
        ]
    }
}

pub fn xr_camera_head_sync(
    views: ResMut<crate::resources::XrViews>,
    mut query: Query<(&mut Transform, &XrCameraType, &mut XRProjection)>,
) {
    let mut f = || -> Option<()> {
        //TODO calculate HMD position
        for (mut transform, camera_type, mut xr_projection) in query.iter_mut() {
            let view_idx = match camera_type {
                XrCameraType::Xr(eye) => *eye as usize,
                XrCameraType::Flatscreen => return None,
            };
            let v = views.lock().unwrap();
            let view = v.get(view_idx)?;
            xr_projection.fov = view.fov;
            transform.rotation = view.pose.orientation.to_quat();
            transform.translation = view.pose.position.to_vec3();
        }
        Some(())
    };
    let _ = f();
}
