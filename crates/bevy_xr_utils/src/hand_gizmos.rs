use bevy::{prelude::*, transform::TransformSystem};
use bevy_xr::hands::{HandBone, HandBoneRadius};
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

fn gizmo_color(bone: &HandBone) -> Color {
    match bone {
        HandBone::Palm => Color::WHITE,
        HandBone::Wrist => Color::GRAY,
        b if b.is_thumb() => Color::RED,
        b if b.is_index() => Color::ORANGE,
        b if b.is_middle() => Color::YELLOW,
        b if b.is_ring() => Color::GREEN,
        b if b.is_little() => Color::BLUE,
        // should be impossible to hit
        _ => Color::rgb(1.0, 0.0, 1.0),
    }
}
