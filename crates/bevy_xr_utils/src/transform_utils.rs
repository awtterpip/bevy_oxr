use bevy::prelude::*;
use bevy_mod_openxr::{
    helper_traits::{ToQuat, ToVec3},
    resources::OxrViews,
};
use bevy_mod_xr::session::XrTrackingRoot;

pub struct TransformUtilitiesPlugin;

impl Plugin for TransformUtilitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SnapToRotation>();
        app.add_event::<SnapToPosition>();
        app.add_systems(PostUpdate, handle_transform_events);
    }
}

//events
#[derive(Event, Debug)]
pub struct SnapToRotation(pub Quat);

#[derive(Event, Debug)]
pub struct SnapToPosition(pub Vec3);

pub fn handle_transform_events(
    mut root_query: Query<&mut Transform, With<XrTrackingRoot>>,
    views: ResMut<OxrViews>,
    mut position_reader: EventReader<SnapToPosition>,
    mut rotation_reader: EventReader<SnapToRotation>,
) {
    let result = root_query.single_mut();
    match result {
        Ok(mut root_transform) => {
            let view = views.first();
            match view {
                Some(view) => {
                    //we want the view translation with a height of zero for a few calculations
                    let mut view_translation = view.pose.position.to_vec3();
                    view_translation.y = 0.0;

                    //position
                    for position in position_reader.read() {
                        root_transform.translation =
                            position.0 - root_transform.rotation.mul_vec3(view_translation);
                    }

                    //rotation
                    let root_local = root_transform.translation;
                    let hmd_global =
                        root_transform.rotation.mul_vec3(view_translation) + root_local;
                    let view_rot = view.pose.orientation.to_quat();
                    let root_rot = root_transform.rotation;
                    let view_global_rotation = root_rot.mul_quat(view_rot).normalize();
                    let (global_view_yaw, _pitch, _roll) =
                        view_global_rotation.to_euler(bevy::math::EulerRot::YXZ);
                    let up = Vec3::Y;
                    for rotation in rotation_reader.read() {
                        let (target_yaw, _pitch, _roll) =
                            rotation.0.normalize().to_euler(bevy::math::EulerRot::YXZ);
                        let diff_yaw = target_yaw - global_view_yaw;

                        //build a rotation quat?
                        let rotation_quat = Quat::from_axis_angle(up, diff_yaw);
                        //apply rotation this works
                        root_transform.rotate_around(hmd_global, rotation_quat);
                    }
                }
                None => debug!("error getting first view"),
            }
        }
        Err(_) => debug!("error getting root transform"),
    }
}
