pub mod actions;
pub mod controllers;
pub mod debug_gizmos;
pub mod hand_poses;
pub mod hands;
pub mod interactions;
pub mod oculus_touch;
pub mod prototype_locomotion;
pub mod trackers;
pub mod xr_camera;

use crate::resources::{XrInstance, XrSession};
use crate::xr_init::{xr_only, XrCleanup, XrPostSetup, XrPreSetup, XrSetup};
use crate::xr_input::oculus_touch::setup_oculus_controller;
use crate::xr_input::xr_camera::{xr_camera_head_sync, Eye, XRProjection, XrCameraBundle};
use crate::{locate_views, xr_wait_frame};
use bevy::app::{App, PostUpdate, Startup};
use bevy::ecs::entity::Entity;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::log::{info, warn};
use bevy::math::Vec2;
use bevy::prelude::{BuildChildren, Component, Deref, DerefMut, IntoSystemConfigs, Resource};
use bevy::prelude::{Commands, Plugin, PreUpdate, Quat, Res, SpatialBundle, Update, Vec3};
use bevy::render::camera::CameraProjectionPlugin;
use bevy::render::extract_component::ExtractComponentPlugin;
use bevy::render::view::{update_frusta, VisibilitySystems};
use bevy::transform::TransformSystem;
use bevy::utils::HashMap;
use openxr::Binding;

use self::actions::{setup_oxr_actions, XrActionsPlugin};
use self::oculus_touch::{
    init_subaction_path, post_action_setup_oculus_controller, ActionSets, OculusController,
};
use self::trackers::{
    adopt_open_xr_trackers, update_open_xr_controllers, OpenXRLeftEye, OpenXRRightEye,
    OpenXRTrackingRoot,
};
use self::xr_camera::{/* GlobalTransformExtract, TransformExtract, */ XrCamera};

#[derive(Copy, Clone)]
pub struct XrInputPlugin;
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Component)]
pub enum Hand {
    Left,
    Right,
}

impl Plugin for XrInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(XrPostSetup, post_action_setup_oculus_controller);
        app.add_systems(XrSetup, setup_oculus_controller);
        app.add_systems(XrCleanup, cleanup_oculus_controller);
        //adopt any new trackers
        app.add_systems(PreUpdate, adopt_open_xr_trackers.run_if(xr_only()));
        // app.add_systems(PreUpdate, action_set_system.run_if(xr_only()));
        //update controller trackers
        app.add_systems(Update, update_open_xr_controllers.run_if(xr_only()));
        app.add_systems(XrPreSetup, init_subaction_path);
        app.add_systems(XrSetup, setup_xr_root);
        app.add_systems(XrCleanup, cleanup_xr_root);
    }
}

fn cleanup_oculus_controller(mut commands: Commands) {
    commands.remove_resource::<OculusController>();
}

fn cleanup_xr_root(
    mut commands: Commands,
    tracking_root_query: Query<Entity, With<OpenXRTrackingRoot>>,
) {
    for e in &tracking_root_query {
        commands.entity(e).despawn_recursive();
    }
}
fn setup_xr_root(
    mut commands: Commands,
    tracking_root_query: Query<Entity, With<OpenXRTrackingRoot>>,
) {
    if tracking_root_query.get_single().is_err() {
        info!("Creating XrTrackingRoot!");
        commands.spawn((SpatialBundle::default(), OpenXRTrackingRoot));
    }
}

// pub fn action_set_system(action_sets: Res<ActionSets>, session: Res<XrSession>) {
//     let mut active_action_sets = vec![];
//     for i in &action_sets.0 {
//         active_action_sets.push(openxr::ActiveActionSet::new(i));
//     }
//     //info!("action sets: {:#?}", action_sets.0.len());
//     if let Err(err) = session.sync_actions(&active_action_sets) {
//         warn!("{}", err);
//     }
// }

pub trait Vec2Conv {
    fn to_vec2(&self) -> Vec2;
}

impl Vec2Conv for openxr::Vector2f {
    fn to_vec2(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
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
