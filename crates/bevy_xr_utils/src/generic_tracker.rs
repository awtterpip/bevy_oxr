use bevy_app::{App, Plugin, PostUpdate};
use bevy_color::palettes::css;
use bevy_ecs::{component::Component, query::With, schedule::IntoScheduleConfigs as _, system::Query};
use bevy_gizmos::gizmos::Gizmos;
use bevy_transform::{TransformSystems, components::{GlobalTransform, Transform}};

#[derive(Clone, Copy, Component)]
#[require(Transform)]
pub struct GenericTracker;

pub struct GenericTrackerGizmoPlugin;

impl Plugin for GenericTrackerGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, draw_gizmos.after(TransformSystems::Propagate));
    }
}

fn draw_gizmos(query: Query<&GlobalTransform, With<GenericTracker>>, mut gizmos: Gizmos) {
    for transform in query {
        gizmos.axes(*transform, 0.05);
        gizmos.sphere(transform.to_isometry(), 0.05, css::PINK);
    }
}
