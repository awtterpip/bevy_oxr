use bevy::color::palettes::css;
use bevy::{prelude::*, transform::TransformSystem};
use bevy_mod_xr::hands::{HandBone, HandBoneRadius};
use bevy_mod_xr::spaces::XrSpaceLocationFlags;
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
    query: Query<(
        &GlobalTransform,
        &HandBone,
        &HandBoneRadius,
        &XrSpaceLocationFlags,
    )>,
) {
    for (transform, bone, radius, flags) in &query {
        if (!flags.position_tracked) || (!flags.rotation_tracked) {
            continue;
        }
        let pose = transform.compute_transform();
        let pose = Isometry3d {
            translation: pose.translation.into(),
            rotation: pose.rotation,
        };
        gizmos.sphere(pose, **radius, gizmo_color(bone));
    }
}

fn gizmo_color(bone: &HandBone) -> Srgba {
    match bone {
        HandBone::Palm => css::WHITE,
        HandBone::Wrist => css::GRAY,
        b if b.is_thumb() => css::RED,
        b if b.is_index() => css::ORANGE,
        b if b.is_middle() => css::YELLOW,
        b if b.is_ring() => Srgba::rgb(0.0, 1.0, 0.0),
        b if b.is_little() => css::BLUE,
        // should be impossible to hit
        _ => Srgba::rgb(1.0, 0.0, 1.0),
    }
}
