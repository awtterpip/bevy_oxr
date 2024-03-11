use bevy::app::{App, Plugin};
use bevy::core_pipeline::core_3d::graph::Core3d;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::reflect::ReflectComponent;
use bevy::math::{Mat4, Vec3A};
use bevy::reflect::Reflect;
use bevy::render::camera::{
    Camera, CameraMainTextureUsages, CameraProjection, CameraProjectionPlugin, CameraRenderGraph,
    Exposure,
};
use bevy::render::primitives::Frustum;
use bevy::render::view::{ColorGrading, VisibleEntities};
use bevy::transform::components::{GlobalTransform, Transform};

pub struct XrCameraPlugin;

impl Plugin for XrCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraProjectionPlugin::<XrProjection>::default());
    }
}

#[derive(Clone, Copy, Component, Reflect, Debug)]
#[reflect(Component)]
pub struct XrProjection {
    pub projection_matrix: Mat4,
    pub near: f32,
    pub far: f32,
}

impl Default for XrProjection {
    fn default() -> Self {
        Self {
            near: 0.1,
            far: 1000.,
            projection_matrix: Mat4::IDENTITY,
        }
    }
}

/// Marker component for an XR view. It is the backends responsibility to update this.
#[derive(Clone, Copy, Component, Debug, Default)]
pub struct XrView;

impl CameraProjection for XrProjection {
    fn get_projection_matrix(&self) -> Mat4 {
        self.projection_matrix
    }

    fn update(&mut self, _width: f32, _height: f32) {}

    fn far(&self) -> f32 {
        self.far
    }

    // TODO calculate this properly
    fn get_frustum_corners(&self, _z_near: f32, _z_far: f32) -> [Vec3A; 8] {
        Default::default()
    }
}

#[derive(Bundle)]
pub struct XrCameraBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: XrProjection,
    pub visible_entities: VisibleEntities,
    pub frustum: Frustum,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub camera_3d: Camera3d,
    pub tonemapping: Tonemapping,
    pub dither: DebandDither,
    pub color_grading: ColorGrading,
    pub exposure: Exposure,
    pub main_texture_usages: CameraMainTextureUsages,
    pub view: XrView,
}

impl Default for XrCameraBundle {
    fn default() -> Self {
        Self {
            camera_render_graph: CameraRenderGraph::new(Core3d),
            camera: Default::default(),
            projection: Default::default(),
            visible_entities: Default::default(),
            frustum: Default::default(),
            transform: Default::default(),
            global_transform: Default::default(),
            camera_3d: Default::default(),
            tonemapping: Default::default(),
            color_grading: Default::default(),
            exposure: Default::default(),
            main_texture_usages: Default::default(),
            dither: DebandDither::Enabled,
            view: XrView,
        }
    }
}
