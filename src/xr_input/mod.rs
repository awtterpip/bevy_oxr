pub mod controllers;
pub mod oculus_touch;
pub mod xr_camera;

use crate::resources::XrSession;
use crate::xr_begin_frame;
use crate::xr_input::controllers::XrControllerType;
use crate::xr_input::oculus_touch::{setup_oculus_controller, ActionSets};
use crate::xr_input::xr_camera::{xr_camera_head_sync, Eye, XRProjection, XrCameraBundle};
use bevy::app::{App, PostUpdate, Startup};
use bevy::log::warn;
use bevy::prelude::IntoSystemConfigs;
use bevy::prelude::{Commands, Plugin, PreUpdate, Quat, Res, Vec3};
use bevy::render::camera::CameraProjectionPlugin;
use bevy::render::view::{update_frusta, VisibilitySystems};
use bevy::transform::TransformSystem;

#[derive(Copy, Clone)]
pub struct OpenXrInput {
    pub controller_type: XrControllerType,
}
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Hand {
    Left,
    Right,
}
impl OpenXrInput {
    pub fn new(controller_type: XrControllerType) -> Self {
        Self { controller_type }
    }
}

impl Plugin for OpenXrInput {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraProjectionPlugin::<XRProjection>::default());
        match self.controller_type {
            XrControllerType::OculusTouch => {
                app.add_systems(Startup, setup_oculus_controller);
            }
        }
        app.add_systems(PreUpdate, action_set_system);
        app.add_systems(PreUpdate, xr_camera_head_sync.after(xr_begin_frame));
        app.add_systems(
            PostUpdate,
            update_frusta::<XRProjection>
                .after(TransformSystem::TransformPropagate)
                .before(VisibilitySystems::UpdatePerspectiveFrusta),
        );
        app.add_systems(Startup, setup_xr_cameras);
    }
}

fn setup_xr_cameras(mut commands: Commands) {
    commands.spawn(XrCameraBundle::new(Eye::Right));
    commands.spawn(XrCameraBundle::new(Eye::Left));
}

fn action_set_system(action_sets: Res<ActionSets>, session: Res<XrSession>) {
    let mut active_action_sets = vec![];
    for i in &action_sets.0 {
        active_action_sets.push(openxr::ActiveActionSet::new(i));
    }
    //info!("action sets: {:#?}", action_sets.0.len());
    match session.sync_actions(&active_action_sets) {
        Err(err) => {
            warn!("{}", err);
        }
        _ => {}
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
