use core::panic;

use bevy::app::{App, Plugin, PostUpdate};
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::{Component, StorageType};
use bevy::ecs::reflect::ReflectComponent;
use bevy::ecs::schedule::IntoSystemConfigs;
use bevy::math::{Mat4, Vec3A};
use bevy::pbr::{PbrPlugin, PbrProjectionPlugin};
use bevy::prelude::{Projection, SystemSet};
use bevy::reflect::std_traits::ReflectDefault;
use bevy::reflect::Reflect;
use bevy::render::camera::{CameraProjection, CameraProjectionPlugin};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::view::{update_frusta, VisibilitySystems};
use bevy::transform::TransformSystem;

use crate::session::XrTracker;

pub struct XrCameraPlugin;

impl Plugin for XrCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraProjectionPlugin::<XrProjection>::default());
        app.add_systems(
            PostUpdate,
            update_frusta::<XrProjection>
                .after(TransformSystem::TransformPropagate)
                .before(VisibilitySystems::UpdateFrusta),
        );
        if app.is_plugin_added::<PbrPlugin>() {
            app.add_plugins(PbrProjectionPlugin::<XrProjection>::default());
        }
        app.add_plugins((
            ExtractComponentPlugin::<XrProjection>::default(),
            ExtractComponentPlugin::<XrCamera>::default(),
        ));
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug, Hash, SystemSet)]
pub struct XrViewInit;

#[derive(Debug, Clone, Reflect, ExtractComponent)]
#[reflect(Component, Default)]
pub struct XrProjection {
    pub fov: XrFov,
    pub near: f32,
}
impl Component for XrProjection {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_component_hooks(hooks: &mut bevy::ecs::component::ComponentHooks) {
        hooks.on_add(|mut world, entity, _| {
            world.commands().entity(entity).remove::<Projection>();
        });
    }
}

impl Default for XrProjection {
    fn default() -> Self {
        Self {
            near: 0.1,
            fov: XrFov::default(),
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Reflect)]
pub struct XrFov {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
}

/// Marker component for an XR view. It is the backends responsibility to update this.
#[derive(Clone, Copy, Component, ExtractComponent, Debug, Default)]
#[require(Camera3d, XrProjection, XrTracker)]
pub struct XrCamera(pub u32);

impl CameraProjection for XrProjection {
    fn update(&mut self, _width: f32, _height: f32) {}

    fn far(&self) -> f32 {
        let matrix = self.get_clip_from_view();
        matrix.to_cols_array()[14] / (matrix.to_cols_array()[10] + 1.0)
    }

    fn get_frustum_corners(&self, z_near: f32, z_far: f32) -> [Vec3A; 8] {
        let near = z_near.abs();
        let far = z_far.abs();

        let tan_left = self.fov.left.tan();
        let tan_right = self.fov.right.tan();
        let tan_up = self.fov.up.tan();
        let tan_down = self.fov.down.tan();

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

    fn get_clip_from_view(&self) -> Mat4 {
        calculate_projection(self.near, self.fov)
    }

    fn get_clip_from_view_for_sub(&self, _sub_view: &bevy::render::camera::SubCameraView) -> Mat4 {
        panic!("sub view not supported for xr camera");
    }
}

fn calculate_projection(near_z: f32, fov: XrFov) -> Mat4 {
    //  symmetric perspective for debugging
    // let x_fov = (self.fov.angle_left.abs() + self.fov.angle_right.abs());
    // let y_fov = (self.fov.angle_up.abs() + self.fov.angle_down.abs());
    // return Mat4::perspective_infinite_reverse_rh(y_fov, x_fov / y_fov, self.near);

    let is_vulkan_api = false; // FIXME wgpu probably abstracts this
    let far_z = -1.; //   use infinite proj
                     // let far_z = self.far;

    let tan_angle_left = fov.left.tan();
    let tan_angle_right = fov.right.tan();

    let tan_angle_down = fov.down.tan();
    let tan_angle_up = fov.up.tan();

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
