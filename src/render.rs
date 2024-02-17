use bevy::ecs::system::Resource;
use bevy::math::Mat4;
use bevy::prelude::{Deref, DerefMut};
use bevy::render::camera::{RenderTarget, Viewport};

use crate::types::Pose;

pub(crate) const XR_TEXTURE_VIEW_INDEX: u32 = 1208214591;

#[derive(Debug, Clone)]
pub struct XrView {
    pub projection_matrix: Mat4,
    pub pose: Pose,
    pub render_target: RenderTarget,
    pub view_port: Option<Viewport>,
}

#[derive(Deref, DerefMut, Default, Debug, Clone, Resource)]
pub struct XrViews(#[deref] pub Vec<XrView>);
