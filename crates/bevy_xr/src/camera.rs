use core::panic;

use bevy_app::{App, Plugin};
use bevy_camera::{Camera3d, CameraProjection};
use bevy_ecs::{component::Component, schedule::SystemSet};
use bevy_math::{Mat4, Vec3A, Vec4};
// use bevy::prelude::SystemSet;
#[cfg(feature = "reflect")]
use bevy_reflect::std_traits::ReflectDefault;
#[cfg(feature = "reflect")]
use bevy_reflect::Reflect;
use bevy_render::extract_component::{ExtractComponent, ExtractComponentPlugin};

use crate::session::XrTracker;

pub struct XrCameraPlugin;

impl Plugin for XrCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<XrCamera>::default());
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, SystemSet)]
pub struct XrViewInit;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "reflect", derive(Reflect))]
#[cfg_attr(feature = "reflect", reflect(Default))]
pub struct XrProjection {
    pub projection_matrix: Mat4,
    pub near: f32,
}

impl Default for XrProjection {
    fn default() -> Self {
        Self {
            near: 0.1,
            projection_matrix: Mat4::IDENTITY,
        }
    }
}

/// Marker component for an XR view. It is the backends responsibility to update this.
#[derive(Clone, Copy, Component, ExtractComponent, Debug, Default)]
#[require(Camera3d, XrTracker)]
pub struct XrCamera(pub u32);

impl CameraProjection for XrProjection {
    fn update(&mut self, _width: f32, _height: f32) {}

    fn far(&self) -> f32 {
        self.projection_matrix.to_cols_array()[14]
            / (self.projection_matrix.to_cols_array()[10] + 1.0)
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        fn normalized_corner(inverse_matrix: &Mat4, near: f32, ndc_x: f32, ndc_y: f32) -> Vec3A {
            let clip_pos = Vec4::new(ndc_x * near, ndc_y * near, near, near);
            // I don't know why multiplying the Z axis by -1 is necessary.
            // As far as I can tell from (likely my incorrect understanding of the code),
            // PerspectiveProjection::get_frustum_corners() has the Z axis inverted??
            Vec3A::from_vec4(inverse_matrix.mul_vec4(clip_pos)) / near * Vec3A::new(1., 1., -1.)
        }

        let inv = self.projection_matrix.inverse();
        let norm_br = normalized_corner(&inv, self.near, 1., -1.);
        let norm_tr = normalized_corner(&inv, self.near, 1., 1.);
        let norm_tl = normalized_corner(&inv, self.near, -1., 1.);
        let norm_bl = normalized_corner(&inv, self.near, -1., -1.);

        [
            norm_br * z_near,
            norm_tr * z_near,
            norm_tl * z_near,
            norm_bl * z_near,
            norm_br * z_far,
            norm_tr * z_far,
            norm_tl * z_far,
            norm_bl * z_far,
        ]
    }

    fn get_clip_from_view(&self) -> Mat4 {
        self.projection_matrix
    }

    fn get_clip_from_view_for_sub(&self, _sub_view: &bevy_camera::SubCameraView) -> Mat4 {
        panic!("sub view not supported for xr camera");
    }
}

#[doc(hidden)]
#[derive(Clone, Copy, Debug)]
pub struct Fov {
    pub angle_left: f32,
    pub angle_right: f32,
    pub angle_down: f32,
    pub angle_up: f32,
}

/// Calculates an asymmetrical perspective projection matrix for XR rendering. This API is for internal use only.
#[doc(hidden)]
pub fn calculate_projection(near_z: f32, fov: Fov) -> Mat4 {
    //  symmetric perspective for debugging
    // let x_fov = (self.fov.angle_left.abs() + self.fov.angle_right.abs());
    // let y_fov = (self.fov.angle_up.abs() + self.fov.angle_down.abs());
    // return Mat4::perspective_infinite_reverse_rh(y_fov, x_fov / y_fov, self.near);

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
    let tan_angle_height = tan_angle_up - tan_angle_down;

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

#[cfg(test)]
mod tests {
    use std::f32::{self, consts::PI};

    use bevy_math::{Mat4,Vec3A};
    use bevy_camera::{CameraProjection, PerspectiveProjection};

    const TEST_VALUES: &[(f32, f32)] = &[(0.5, 100.0), (50.0, 200.0)];

    use super::XrProjection;

    /// Test that calculate_projection works correctly for symmetrical FOV parameters, by comparing against glam.
    #[test]
    fn test_calculate_symmetrical() {
        let half_fov_y = PI * 0.25;
        let aspect = 1.;
        let fov = super::Fov {
            angle_left: -half_fov_y * aspect,
            angle_right: half_fov_y * aspect,
            angle_down: -half_fov_y,
            angle_up: half_fov_y,
        };

        let near = 0.1;

        let matrix = super::calculate_projection(near, fov);
        let control = Mat4::perspective_infinite_reverse_rh(2. * half_fov_y, aspect, near);

        assert_eq!(matrix, control);
    }

    /// Test that XrProjection::get_frustum_corners works correctly for a symmetrical projection matrix,
    /// by comparing against Bevy's PerspectiveProjection.
    #[test]
    fn test_get_frustum_corners_symmetrical() {
        let control_proj = PerspectiveProjection {
            near: 0.1,
            ..Default::default()
        };

        let projection = XrProjection {
            near: control_proj.near,
            projection_matrix: control_proj.get_clip_from_view(),
        };

        for (near, far) in TEST_VALUES {
            let corners = projection.get_frustum_corners(*near, *far);
            let control_corners = control_proj.get_frustum_corners(*near, *far);

            assert!(equals_in_tolerance(&corners, &control_corners));
        }
    }

    /// Test that XrProjection::get_frustum_corners works correctly for a symmetrical projection matrix with a non-infinite far plane,
    /// by comparing against Bevy's PerspectiveProjection.
    #[test]
    fn test_get_frustum_corners_symmetrical_far_plane() {
        let control_proj = PerspectiveProjection {
            near: 0.1,
            ..Default::default()
        };

        let projection = XrProjection {
            near: control_proj.near,
            // Invert far and near plane to create reverse-Z far-plane perspective matrix.
            projection_matrix: Mat4::perspective_rh(
                control_proj.fov,
                control_proj.aspect_ratio,
                control_proj.far,
                control_proj.near,
            ),
        };

        for (near, far) in TEST_VALUES {
            let corners = projection.get_frustum_corners(*near, *far);
            let control_corners = control_proj.get_frustum_corners(*near, *far);

            assert!(equals_in_tolerance(&corners, &control_corners));
        }
    }

    /// Test that XrProjection::get_frustum_corners works correctly for an asymmetrical projection matrix,
    /// by comparing against an implementation similar to that of Bevy's PerspectiveProjection.
    #[test]
    fn test_get_frustum_corners_asymmetrical() {
        let fov = super::Fov {
            angle_left: -PI * 0.33,
            angle_right: PI * 0.25,
            angle_down: -PI * 0.25,
            angle_up: PI * 0.25,
        };

        let near = 0.1;

        let projection = XrProjection {
            near,
            projection_matrix: super::calculate_projection(near, fov),
        };

        for (near, far) in TEST_VALUES {
            let corners = projection.get_frustum_corners(*near, *far);
            let control_corners = get_frustum_corners_asymmetrical_control(fov, *near, *far);

            assert!(equals_in_tolerance(&corners, &control_corners));
        }
    }

    const TOLERANCE: f32 = 0.0001;

    /// Check whether two sets of frustum corner values are "close enough" within a tolerance.
    fn equals_in_tolerance(a: &[Vec3A; 8], b: &[Vec3A; 8]) -> bool {
        a.iter()
            .zip(b.iter())
            .all(|(a, b)| (a - b).abs().max_element() < TOLERANCE)
    }

    fn get_frustum_corners_asymmetrical_control(
        fov: super::Fov,
        z_near: f32,
        z_far: f32,
    ) -> [Vec3A; 8] {
        let near = z_near.abs();
        let far = z_far.abs();

        let tan_left = fov.angle_left.tan();
        let tan_right = fov.angle_right.tan();
        let tan_up = fov.angle_up.tan();
        let tan_down = fov.angle_down.tan();

        [
            Vec3A::new(tan_right * near, tan_down * near, z_near), // Bottom-right
            Vec3A::new(tan_right * near, tan_up * near, z_near),   // Top-right
            Vec3A::new(tan_left * near, tan_up * near, z_near),    // Top-left
            Vec3A::new(tan_left * near, tan_down * near, z_near),  // Bottom-left
            Vec3A::new(tan_right * far, tan_down * far, z_far),    // Bottom-right
            Vec3A::new(tan_right * far, tan_up * far, z_far),      // Top-right
            Vec3A::new(tan_left * far, tan_up * far, z_far),       // Top-left
            Vec3A::new(tan_left * far, tan_down * far, z_far),     // Bottom-left
        ]
    }
}
