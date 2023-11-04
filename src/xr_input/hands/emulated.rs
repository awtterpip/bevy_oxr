use bevy::prelude::*;

use crate::xr_input::Hand;

use super::HandBone;

#[derive(Deref, DerefMut, Resource)]
pub struct EmulatedHandPose(pub Box<dyn Fn(Hand, HandBone) -> (Vec3, Quat) + Send + Sync>);

pub struct EmulatedHandsPlugin;

impl Plugin for EmulatedHandsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_hand_skeleton_from_emulated);
    }
}

pub(crate) fn update_hand_skeleton_from_emulated() {}
