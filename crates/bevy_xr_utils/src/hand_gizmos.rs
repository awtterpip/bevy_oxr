use bevy::color::palettes::css;
use bevy::{prelude::*, transform::TransformSystem};
use bevy_mod_xr::hands::{HandBone, HandBoneRadius};
pub struct HandGizmosPlugin;
impl Plugin for HandGizmosPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            draw_hand_gizmos.after(TransformSystem::TransformPropagate),
        );
    }
}
fn draw_hand_gizmos(
    mut gizmos: Gizmos,
    query: Query<(&GlobalTransform, &HandBone, &HandBoneRadius)>,
) {
    for (transform, bone, radius) in &query {
        let pose = transform.compute_transform();
        gizmos.sphere(pose.translation, pose.rotation, **radius, gizmo_color(bone));
    }
}

fn gizmo_color(bone: &HandBone) -> Srgba {
    match bone {
        HandBone::Palm => css::WHITE,
        HandBone::Wrist => css::GRAY,
        b if b.is_thumb() => css::RED,
        b if b.is_index() => css::ORANGE,
        b if b.is_middle() => css::YELLOW,
        b if b.is_ring() => css::GREEN,
        b if b.is_little() => css::BLUE,
        // should be impossible to hit
        _ => Srgba::rgb(1.0, 0.0, 1.0),
    }
}
